"""
22.12.2014
by real
http://www.freedomlayer.org

Checking the idea of Landmarks Navigation with random walk.
The Landmarks are a set of nodes that are known to every node in the network.
Every node x keeps the shortest path to each of the landmarks.

For every node x, the length of the shortest paths to each of the landmarks
create a coordinate. This coordinate is x's address in the network.

Given a set of coordinates of a destination node, we want to be able to
navigate to that node, given only the information of the current coordinate at
every location in the network.
"""

import math
import random
import collections

import numpy as np

import networkx as nx

#############[Array operations]######################

def avg(lst):
    """
    Calculate the average of an array of numbers.
    """
    assert len(lst) > 0,"List must contain something"
    return sum(lst)/len(lst)


def normalize_prob(lst):
    """
    Normalize a list of numbers so that the sum will be 1.
    """
    total_sum = sum(lst)
    assert total_sum > 0,"sum should be positive"
    return [x/total_sum for x in lst]

############################################################


class GraphCoords(object):
    def __init__(self,graph,k):

        # The network's layout is given as a networkx graph:
        self.graph = graph

        # We expect that the amount of landmarks will be less than the amount
        # of nodes in the network:
        assert k < self.graph.number_of_nodes(),\
                "We can't have more landmarks than nodes."

        # k is the number of landmarks (Special nodes).
        self.k = k

        # Initialize landmarks and dists:
        self.landmarks = random.sample(self.graph.nodes(),self.k)
        self.dists = None

        # Calculate distances from every landmark to all the nodes in the
        # graph (We can derive all the coordinates from this information):
        self.calc_dists()

    def calc_dists(self):
        """
        Calculate all distances from every landmark to all the parties in the
        graph.
        """
        # Should be done only once: 
        assert self.dists is None,"calc_dists should be invoked only once."

        # Initialize the distances dictionary.
        # Note that this dictionary has keys of the form (ld,nd),
        # where ld is a landmark node, and nd is any node. ((nd,ld) will not
        # work)
        self.dists = {}

        # For every landmark ld, we calculate the distances from ld to each of
        # the nodes in the graph. Those distances will later be used as
        # coordinates for all the nodes in the graph.
        for ld in self.landmarks:
            lengths = nx.single_source_dijkstra_path_length(self.graph,ld)
            for nd in self.graph.nodes():
                self.dists[(ld,nd)]=lengths[nd]

    def calc_dist(self,n1,n2):
        """
        Calculate real distance between two nodes in the graph.
        Could be slow for big graphs.
        """
        return nx.shortest_path_length(self.graph,n1,n2)

    def get_coord(self,nd):
        """
        Get "network coordinate" for a node nd.
        The networks coordinate is a list of shortest distances to each of the
        network landmarks.
        """
        coord = []
        for ld in self.landmarks:
            coord.append(self.dists[(ld,nd)])
        return tuple(coord)


    def all_diffs(self,x,y):
        """
        Calculate all diffs of the form |d(a,y) - d(a,x)| where a is a
        landmark.
        """
        def d(ld,x):
            """
            Distance between a landmark and a vertex.
            """
            return self.dists[(ld,x)]

        return [abs(d(ld,x) - d(ld,y)) \
                for ld in self.landmarks]


    def obs_max_dist(self,x,y):
        """
        Calculates min{ld \in landmarks}{|d(ld,x) - d(ld,y)|}
        """
        return max(self.all_diffs(x,y))


    def random_walk(self,src,dst,cnt_visited_nodes,base=150):
        """
        Try to travel from src to dst
        The "closest" neighbor to dst has the highest probability.
        """
        def e(x,y):
            """
            Distance metric
            """
            return self.obs_max_dist(x,y)

        def gen_weight_by_dist(cur_dist,new_dist):
            """
            Generate a weight for moving from cur_dist to new_dist
            """
            return base**(cur_dist - new_dist)

        # Counter for amount of steps so far in the random walk:
        num_hops = 0
        # x is the current node in the random walk. We begin from the src node.
        x = src
        # Add x to visited nodes counter:
        cnt_visited_nodes[x] += 1
        # Current distance from the destination:
        cur_dist = e(x,dst)

        while x != dst:
            neighbours = []
            for nei in self.graph.neighbors_iter(x):
                # Append neighbour and obs distance:
                neighbours.append((nei,e(nei,dst)))

            # Get weights according to obs_distance:
            weights = [gen_weight_by_dist(cur_dist,dist) for (nei,dist) in neighbours]
            # Normalize the calculated weights to be a probability vector.
            # (Probabilities vector always has a some of 1)
            probs = normalize_prob(weights)

            # Choose the next neighbour. Neighbours which are closer to dst
            # Get a better probability to be chosen.
            index = np.random.choice(len(neighbours),p=probs)
            x,dist = neighbours[index]
            cur_dist = dist

            num_hops += 1

            # Add x to visited nodes counter:
            cnt_visited_nodes[x] += 1


        return num_hops

    def get_avg_num_hops(self,num_messages=0x30,base=150):
        """
        Get the average amount of hops needed to send a message using the
        random walk method. We approximate this number by sending a few
        messages in between randomly chosen pairs of nodes, and averaging the
        amount of hops.
        """

        # A list to keep the amount of hops used for each message delivery:
        hops_list = []

        # Initialize counter for visited nodes. This should measure the load
        # on specific nodes in the network.
        cnt_visited_nodes = collections.Counter()

        for i in range(num_messages):
            # Obtain a random pair of nodes: (x,y are different)
            x,y = random.sample(self.graph.nodes(),2)
            # Start a random walk from x, in attempt to find y:
            num_hops = self.random_walk(x,y,cnt_visited_nodes,base)
            hops_list.append(num_hops)

        # Return the average value for number of hops:
        return avg(hops_list),cnt_visited_nodes

    def count_coords(self):
        """
        Count unique nodes coordinates in the network.
        Returns a counter that contains the amount of times every coordinate
        appear. We hope that every coordinate appears only once.
        """

        cnt_coord = collections.Counter()
        for n in self.graph.nodes_iter():
            coord = self.get_coord(n)
            # Increase coordinate count:
            cnt_coord[coord] += 1

        return cnt_coord

def geo_graph(m,n,d):
    """
    Generate a grid graph of size mXn.
    Then connect every node to d random nodes in the graph.
    Somewhat similar to the idea of a small-world graph.
    """
    # Begin with a grid:
    res_graph = nx.grid_2d_graph(m,n,True)

    # Next connect each node to random nodes:
    for x in res_graph.nodes_iter():
        neis = random.sample(res_graph.nodes(),d)
        # We dont want to connect to ourselves:
        try:
            neis.remove(x)
        except ValueError:
            pass

        # Add the additional edges:
        for nei in neis:
            if not res_graph.has_edge(x,nei):
                res_graph.add_edge(x,nei)

    return res_graph

def gen_grid_graph(i):
    """
    Generate a grid graph with about 2**i nodes.
    """
    n = 2**i
    sn = int(n**(1/2))
    # Redefine amount of nodes:
    return nx.grid_2d_graph(sn,sn)

def gen_gnp_graph(i):
    """
    Generate a gnp random graph with 2**i nodes.
    """
    n = 2**i
    p = 2*i / (2**i)
    return nx.fast_gnp_random_graph(n,p)


