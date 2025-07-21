#pragma once

#include <string>
#include <unordered_map>


namespace TopologicOptimizer::Graph::Essentials
{
    template <typename IDType, typename PayloadType = void>
    class Node {
    public:
        IDType id;
        PayloadType data;

        Node(IDType id, PayloadType data) : id(id), data(data) {}
    };
}