# TÓPICOS ESPECIAIS EM COMPUTAÇÃO II - PROGRAMAÇÃO AVANÇADA

## TRABALHO 6 - FÁBRICAS

O código implementa um programa interativo com um modo `Obstáculos` que permite ao usuário criar obstáculos manualmente (barreiras no grid), `Agente` que permite colocar um ponto de partida e chegada para um agente e então calcula o caminho que ele deve usar (caso seja possível) para chegar ao ponto de chegada em tempo real.

Também é implementado um modo `Aleátorios` que gera vários agentes com aleátorios pontos de partida e chegada. E um modo `benchmark` que faz testes para acompanhar o desempenho do programa.

---

O projeto foi refatorado para implementar o padrão de projeto Fábricas. Separando a lógica de criação de objetos da lógica de uso dos objetos.

Agora o main não instancia objetos diretamente com `::new`, agora isso ocorre através de solicitações a uma fábrica

Fábrica Abstrata: `abstract_factory.rs` Define o conjunto coerente de componentes para uma simulação (SimulationFactory). A CardinalSimulationFactory padrão retorna todos os componentes para a simulação 4-direções.

Factory Method `agent_factory.rs` Implementa o trait AgentFactory com as fábricas concretas (BlueAgentFactory, RedAgentFactory). Permite definir a cor e características do agente no momento da criação.

Factory Method `pathfinding_factory.rs` Implementa o trait PathfindingAlgorithm com o wrapper AStarCardinal. Permite trocar o algoritmo (ex: para Dijkstra ou A\* 8-direções) facilmente.

Factory Method `grid_factory.rs` Implementa o trait GridFactory com a RectangularGridFactory. Responsável por criar a estrutura do ambiente de simulação.

---

`agent.rs:` Adicionado o campo pub color: Color à estrutura Agent. O construtor Agent::new agora exige que a cor seja fornecida pela fábrica.

`renderer.rs:` A função draw_agents foi modificada para ler a cor diretamente do campo agent.color, garantindo que cada agente seja desenhado com a cor que sua fábrica definiu.

`benchmark.rs:` A função run_benchmark agora recebe um `&dyn PathfindingAlgorithm`, permitindo que o cliente especifique qual algoritmo deve ser testado, mantendo a flexibilidade.

`main.rs:`

-   O loop principal começa instanciando a CardinalSimulationFactory::new().
-   grid, pathfinder e agent_creator são obtidas via métodos da fábrica (factory.create_grid(), factory.create_pathfinder(), etc.), eliminando a dependência de classes concretas.
-   Agentes criados aleatoriamente usam a RedAgentFactory, enquanto agentes criados manualmente com o mouse usam BlueAgentFactory.
