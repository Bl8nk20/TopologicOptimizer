set(Data_reader_LIB_Name
    DataReader)

set(Data_reader_LIB_Headers
    include/${Data_reader_LIB_Name}/DataReader.hpp
)

set(Data_reader_LIB_Sources
    DataReader.cpp
)

add_library(${Data_reader_LIB_Name} 
    STATIC
    ${Data_reader_LIB_Headers} 
    ${Data_reader_LIB_Sources})

target_include_directories( ${Data_reader_LIB_Name} 
    PUBLIC 
    include)