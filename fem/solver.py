import Fem
import FemGui


class SolverManager:
    def __init__(self, doc, analysis):
        self.doc = doc
        self.analysis = analysis

    def setup_calculix_solver(self):
        solver = self.doc.addObject('Fem::FemSolverObjectPython', 'CalculiX')
        import femsolver.calculix
        femsolver.calculix.FemSolverCalculix(solver)
        solver.GeometricalNonlinearity = False
        self.analysis.addObject(solver)
        return solver

    def run_solver(self):
        FemGui.setActiveAnalysis(self.analysis)
        Fem.run(self.analysis.Member[-1])
