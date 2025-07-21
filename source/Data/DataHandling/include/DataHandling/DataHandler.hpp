#ifdef DataHandler
#define DataHandler

#pragma once
// Includes
#include <string>
#include "DataReader/DataReader.hpp"
#include "DataWriter/DataWriter.hpp"

// Namespace
namespace TopologicOptimizer::Data
{
    template <typename T> class DataHandler{
        public:
        T read_file(const std::string& filename) const;
        void write_file(const std::string& filename, const T& new_Mesh) const;

        private:
        TopologicOptimizer::Data::DataReader<T> reader;
        TopologicOptimizer::Data::DataWriter<T> writer;

    };
} // namespace TopologicOptimizer::Data



#endif