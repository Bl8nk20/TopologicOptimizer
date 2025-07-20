# TopologicOptimizer

This shoulb become currently only a standalone Library and maybe console application to run a topologic optimization using C++ and CMake.

## Target

Optimize CAD-Models for weight-reduction, optimization for strength and optics for a better fdm printing experience

## which Design Patterns to use

* Dependency Injection ( Concrete Types for DataWriter, DataReader)
* Singleton for DataHandling ( DataHandler, DataWriter, DataReader)
* abstract Classes for calculation Logic
* Interfaces for Data-Reader and Writer
* Builder Pattern to build Graph of Mesh ?
