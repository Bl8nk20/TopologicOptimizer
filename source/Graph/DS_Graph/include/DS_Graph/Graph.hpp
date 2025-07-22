#pragma once

#include <string>
#include <unordered_map>
#include <vector>

#include "Node.hpp"
#include "Edge.hpp"

namespace TopologicOptimizer::Graph::Essentials
{
    template <typename IDType,
            typename NodePayload = void,
            typename EdgeWeight = double,
            typename EdgePayload = void>
    class Graph {
    public:
        std::unordered_map<IDType, Node<IDType, NodePayload>> nodes;
        std::vector<Edge<IDType, EdgeWeight, EdgePayload>> edges;

        void print() const {

            for (const auto& [id, node] : nodes) {
                std::cout << "Node: " << id << "\n";
            }
            for (const auto& edge : edges) {
                std::cout << "Edge: " << edge.from << " -> " << edge.to
                        << " (weight: " << edge.weight << ")\n";
            }
        }
    };  
}