set(Data_writer_Lib_Name
    DataWriter)

set(Data_writer_Lib_Neaders
    include/${Data_writer_Lib_Name}/DataWriter.hpp
)

set(Data_writer_Lib_Sources
    DataWriter.cpp
)

add_library(${Data_writer_Lib_Name} 
    STATIC
    ${Data_writer_Lib_Neaders} 
    ${Data_writer_Lib_Sources})

target_include_directories( ${Data_writer_Lib_Name} 
    PUBLIC 
    include)