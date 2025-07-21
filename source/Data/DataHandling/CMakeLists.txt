set(Data_handler_LIB_Name
    DataHandling)

set(Data_handler_LIB_Headers
    include/${Data_handler_LIB_Name}/DataHandler.hpp
)

set(Data_handler_LIB_Sources
    DataHandler.cpp
)

add_library(${Data_handler_LIB_Name} 
    STATIC
    ${Data_handler_LIB_Headers} 
    ${Data_handler_LIB_Sources})

target_include_directories( ${Data_handler_LIB_Name} 
    PUBLIC 
    include)