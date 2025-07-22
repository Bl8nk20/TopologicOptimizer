project(Graph LANGUAGES CXX)

# Create the top-level interface target
add_library(Graph INTERFACE)

# Add include directories to the Graph target
target_include_directories(Graph
    INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../include>
    $<INSTALL_INTERFACE:include>
)

# Collect headers for IDE visibility (optional)
file(GLOB_RECURSE GRAPH_HEADERS
    ${CMAKE_CURRENT_SOURCE_DIR}/../include/Graph/Builder_Graph/*.hpp
    ${CMAKE_CURRENT_SOURCE_DIR}/../include/Graph/DS_Graph/*.hpp
)

# Make headers visible in IDEs
target_sources(Graph INTERFACE ${GRAPH_HEADERS})

# Optionally link components to the Graph target
target_link_libraries(Graph INTERFACE DS_Graph Builder_Graph)
