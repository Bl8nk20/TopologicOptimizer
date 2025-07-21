#ifdef DataReader
#define DataReader

#pragma once
// Includes
#include <string>

// Namespace
namespace TopologicOptimizer::Data
{
    template <typename T> class A_DataReader{
        public:
            virtual T read_file(std::string& filepath) const = 0;

    };
} // namespace TopologicOptimizer::Data



#endif