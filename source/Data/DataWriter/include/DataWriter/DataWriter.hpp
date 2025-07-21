#pragma once

#include <string>
#include <fstream>
#include <iostream>

namespace TopologicOptimizer::Data
{
    template <typename T>
    class DataWriter {
    public:
        void write_file(const std::string& filename, const T& data) const;
    };

    template <typename T>
    void DataWriter<T>::write_file(const std::string& filename, const T& data) const {
        std::ofstream out(filename);
        if (!out) {
            std::cerr << "Error when opening " << filename << " . \n";
            return;
        }
        out << data;
    }
}
