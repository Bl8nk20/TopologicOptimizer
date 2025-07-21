#pragma once

// Includes
#include <string>

// Namespace
namespace TopologicOptimizer::Data
{
    template <typename T> class A_DataReader{
        public:
            virtual T read_file(const std::string& filepath) const = 0;
            virtual ~A_DataReader() = default; // Virtueller Destruktor

    };
} // namespace TopologicOptimizer::Data
