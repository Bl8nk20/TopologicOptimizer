class TopologyOptimizer:
    def __init__(self, fem_model, fem_solver):
        self.fem_model = fem_model
        self.fem_solver = fem_solver

    def run(self, iterations=5):
        for i in range(iterations):
            print(f"--- Iteration {i+1} ---")
            self.fem_solver.run()

            # Platzhalter für:
            # - Compliance berechnen
            # - Materialdichte aktualisieren
            # - Visualisierung oder Feedback

            print("Analyse abgeschlossen.")

            # TODO: Update Mesh oder Materialverteilung
