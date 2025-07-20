function(add_git_submodule dir)
    find_package(Git REQUIRED)

    set(FULL_PATH ${CMAKE_SOURCE_DIR}/${dir})

    if(NOT EXISTS ${FULL_PATH}/CMakeLists.txt)
        execute_process(COMMAND ${GIT_EXECUTABLE}
            submodule update --init --recursive -- ${dir}
            WORKING_DIRECTORY ${PROJECT_SOURCE_DIR})
    endif()

    if(EXISTS ${FULL_PATH}/CMakeLists.txt)
        message("Submodule is a CMake Project ${FULL_PATH}")
        add_subdirectory(${dir})
    else()
        message("Submodule is not a CMake Project ${FULL_PATH}")
    endif()
endfunction(add_git_submodule)