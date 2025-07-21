# Build a library called "Graph" from all graph modules
project(Graph LANGUAGES CXX)

file(GLOB_RECURSE GRAPH_HEADERS
    Builder_Graph/*.hpp
    DS_Graph/*.hpp
)

add_library(Graph INTERFACE)

# Header-only: Interface includes
target_include_directories(Graph INTERFACE
    ${CMAKE_CURRENT_SOURCE_DIR}/DS_Graph/include/DS_Graph
    ${CMAKE_CURRENT_SOURCE_DIR}/Builder_Graph/include/Builder_Graph
)

# Header-Dateien in der IDE sichtbar machen
target_sources(Graph INTERFACE ${GRAPH_HEADERS})
