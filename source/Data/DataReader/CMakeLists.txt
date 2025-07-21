set(Data_reader_LIB_Name
    DataReader)

set(Data_reader_LIB_Headers
    include/${Data_reader_LIB_Name}/DataReader.hpp
)

add_library(${Data_reader_LIB_Name} 
    INTERFACE)

target_include_directories(${Data_reader_LIB_Name}
    INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)