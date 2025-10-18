# TÓPICOS ESPECIAIS EM COMPUTAÇÃO II - PROGRAMAÇÃO AVANÇADA

## TRABALHO 3 - ENVOLTÓRIA CONVEXA

O envio consta de:

-   Executável do programa principal `envoltoria-convexa.exe`
-   Notebook Colab contendo o código para os gráficos de benchmark `tarefa_3_gráficos.ipynb`
    -   arquivo .csv que contém os dados de uma execução de benchmark `benchmark_results.csv`
-   Vídeo que demonstra a execução do programa `showcase.mp4`

### RELAÇÃO ENTRE ENVOLTÓRIA CONVEXA E DIAGRAMA DE VORONOI

A relação entre a Envoltória Convexa e o Diagrama de Voronoi é definida através de um conceito chamado dualidade.

    Diagrama de Voronoi: Para um conjunto de pontos P, o Diagrama de Voronoi divide o plano em regiões. Cada região V(p) contém todos os pontos no plano que estão mais próximos do ponto p∈P do que de qualquer outro ponto em P.

    Triangulação de Delaunay: A Triangulação de Delaunay é o grafo dual do Diagrama de Voronoi. Se você criar um vértice para cada região de Voronoi (ou seja, para cada ponto p) e desenhar uma aresta entre dois vértices se suas regiões de Voronoi forem vizinhas (compartilharem uma borda), você obtém a Triangulação de Delaunay.

    A Conexão: A Envoltória Convexa de P é exatamente a fronteira externa da Triangulação de Delaunay de P.

Portanto, a envoltória convexa define o "limite" ou "perímetro" do conjunto de pontos, e todos os triângulos da Triangulação de Delaunay (e, por sua vez, todas as regiões internas do Diagrama de Voronoi) estão contidos dentro dessa envoltória.

### EXISTE DIFERENÇA DE CUSTO COMPUTACIONAL DEPENDENDO DA DISTRIBUIÇÃO DE PONTOS?

A complexidade Big-O de O(n log n) não muda, pois ela é definda pela ordenação, que sempre vai levar O(n log n). Porem, o tempo de execução real vai mudar.

`Melhor caso:` Pontos em um círculo (circle_points). Quase não haverá operações pop(), pois os pontos já estão na ordem correta da envoltória.

`Pior caso:` Pontos aleatórios (random_points). Haverá muitas operações pop() à medida que a envoltória é construída e refinada.

Foi plotado o gráfico de 3 execuções (Tempo vs. N):

    Linha 1: "Aleatórios"

    Linha 2: "Círculo"

    Linha 3: "Retângulo"

Resultado: Todas as três linhas seguiram uma curva O(n log n), mas a linha "Círculo" e "Retângulo" é visivelmente mais rápida do que a linha "Aleatórios".
