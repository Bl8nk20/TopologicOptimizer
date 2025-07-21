#ifdef DataWriter
#define DataWriter

#pragma once
// Includes
#include <string>

// Namespace
namespace TopologicOptimizer::Data
{
    template <typename T> class DataHandler{
        public:
            void write_file(const std::string& filename, const T& new_Mesh) const;
    };
} // namespace TopologicOptimizer::Data



#endif