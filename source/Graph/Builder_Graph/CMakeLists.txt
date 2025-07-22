# Library Name
set(Graph_Builder_Lib_Name Builder_Graph)

# Header files (header-only => nur .hpp)
set(Graph_Builder_Lib_Header
    include/${Graph_Builder_Lib_Name}/Graph_Builder.hpp
)

# Header-only Library – INTERFACE
add_library(${Graph_Builder_Lib_Name} INTERFACE ${Graph_Builder_Lib_Header})

# Include-Verzeichnis bekannt machen
target_include_directories(${Graph_Builder_Lib_Name}
    INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)
