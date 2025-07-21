# Library Name
set(Graph_DS_Lib_Name DS_Graph)

# Header files (header-only => nur .hpp)
set(Graph_DS_Lib_Header
    include/${Graph_DS_Lib_Name}/Edge.hpp
    include/${Graph_DS_Lib_Name}/Node.hpp
    include/${Graph_DS_Lib_Name}/Graph.hpp
    )

# Header-only Library – INTERFACE
add_library(${Graph_DS_Lib_Name} INTERFACE Graph_DS_Lib_Header)

# Include-Verzeichnis bekannt machen
target_include_directories(${Graph_DS_Lib_Name}
    INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)

