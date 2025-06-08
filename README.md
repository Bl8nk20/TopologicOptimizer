# TopologicOptimizer

This should become an external Library to be included in the FreeCAD Environment.

## Questions to Tackle

Where has to be material to assert the function of the piece?

## Target

Optimize CAD-Models for weight-reduction, optimization for strength and optics for a better fdm printing experience

## Possible Input parameters

### external forces

like torque, gravity, general forces or momentum

### construction limitations

like building-area, mounting-points or allowed material type / material amt

## What Inputs are Required

## What Outputs are Desired

## which Design Patterns to use

| Komponente           | Design Pattern       | Zweck                                |
| -------------------- | -------------------- | ------------------------------------ |
| CAD-Backends         | Strategy / Adapter   | Austauschbare CAD-Systeme            |
| Optimierer           | Template Method      | Gemeinsames Optimierungs-Framework   |
| FEM-Solver           | Facade               | Versteckt komplexe Solver-APIs       |
| Konfiguration        | pydantic / Builder   | Strukturierte, überprüfbare Settings |
| GUI / CLI Feedback   | Observer             | Fortschritt sichtbar machen          |
| Schnittstellen/Tests | Dependency Injection | Testbarkeit und Modularität          |


## To-Do's

### Topologic Optimizer

* [ ] Receive CAD-Sketch / Plan from CAD-SW
* [ ] Generate Mesh (Independence of CAD-SW)
* [ ] Apply Forces on Mesh
* [ ] Remove unneccessary Nodes & Edges
* [ ] Iterate until Satisfied
