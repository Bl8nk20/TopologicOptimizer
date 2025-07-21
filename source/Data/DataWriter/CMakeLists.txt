# Library Name
set(Data_writer_LIB_Name DataWriter)

# Header files (header-only => nur .hpp)
set(Data_writer_LIB_Headers
    include/${Data_writer_LIB_Name}/DataWriter.hpp
)

# Header-only Library – INTERFACE
add_library(${Data_writer_LIB_Name} INTERFACE)

# Include-Verzeichnis bekannt machen
target_include_directories(${Data_writer_LIB_Name}
    INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)

