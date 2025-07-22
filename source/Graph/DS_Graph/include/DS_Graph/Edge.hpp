#pragma once

#include <string>
#include <unordered_map>

namespace TopologicOptimizer::Graph::Essentials
{
    template <typename IDType, typename WeightType = double, typename PayloadType = void>
    class Edge {
    public:
        IDType from;
        IDType to;
        WeightType weight;
        PayloadType data;
        Edge(IDType from, IDType to, WeightType weight, PayloadType data)
            : from(from), to(to), weight(weight), data(data) {}
    };
}