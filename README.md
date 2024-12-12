
# Evo: Natural Selection Simulator

Evo is a small  natural selection simulation program written in Rust . It offers a platform to explore the fascinating dynamics of evolution in a simulated environment.

## About

- **Natural Selection Simulation**: Visualize natural selection in a dynamic, simulated ecosystem.

- **Inspiration**: Built upon the concepts presented in [this video](https://www.youtube.com/watch?v=N3tRFayqVtk&t=2575s)

- **Graphics**: using Bevy with Lyon for graphics

#### Gene Structure

Sensory neurons
0 - Lx - East West Location (0 - 1)
1 - Ly - North South Location (0 - 1)

Action neurons
0 - Mx - Move Left Right(-/+) (-1 1)
1 - My - Move Up Down (-/+) (-1 1)

![evo demo](demo.gif)
