#pragma once

namespace TopologicOptimizer::Graph::Builder
{
    template <typename IDType,
            typename NodePayload = void,
            typename EdgeWeight = double,
            typename EdgePayload = void>
    class GraphBuilder {
    public:
        GraphBuilder(GraphType type = GraphType::Directed)
            : graph(type) {}

        GraphBuilder& addNode(const IDType& id, const std::optional<NodePayload>& payload = std::nullopt) {
            if constexpr (std::is_same_v<NodePayload, void>) {
                graph.nodes.emplace(id, Node<IDType>(id));
            } else {
                graph.nodes.emplace(id, Node<IDType, NodePayload>(id, payload.value()));
            }
            return *this;
        }

        GraphBuilder& addEdge(const IDType& from,
                            const IDType& to,
                            const EdgeWeight& weight = EdgeWeight{1},
                            const std::optional<EdgePayload>& payload = std::nullopt) {
            if constexpr (std::is_same_v<EdgePayload, void>) {
                graph.edges.emplace_back(from, to, weight);
            } else {
                graph.edges.emplace_back(from, to, weight, payload.value());
            }

            // Automatisch Knoten hinzufügen, wenn sie noch nicht existieren
            if (graph.nodes.find(from) == graph.nodes.end())
                addNode(from);
            if (graph.nodes.find(to) == graph.nodes.end())
                addNode(to);

            // Bei ungerichtetem Graph beide Richtungen eintragen
            if (graph.type == GraphType::Undirected && from != to) {
                if constexpr (std::is_same_v<EdgePayload, void>) {
                    graph.edges.emplace_back(to, from, weight);
                } else {
                    graph.edges.emplace_back(to, from, weight, payload.value());
                }
            }

            return *this;
        }

        Graph<IDType, NodePayload, EdgeWeight, EdgePayload> build() {
            return graph;
        }

    private:
        Graph<IDType, NodePayload, EdgeWeight, EdgePayload> graph;
    };
}