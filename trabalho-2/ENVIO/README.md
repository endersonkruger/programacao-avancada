
# TÓPICOS ESPECIAIS EM COMPUTAÇÃO II - PROGRAMAÇÃO AVANÇADA

## TRABALHO 2 - DIAGRAMA DE VORONOI

O envio consta de:
- Executável do programa principal "voronoi.exe"
- Executável do benchmark "voronoi-benchmark.exe"
    - arquivo .csv que contém os dados de uma execução de benchmark "benchmark_data.csv"
    - imagem .png que contém o gráfico de uma execução baseada nos dados do .csv "benchmark_graph.png"
- Vídeo que demonstra a execução do programa "showcase.mp4"

### EXISTE DIFERENÇA DE CUSTO COMPUTACIONAL DEPENDENDO DA DISTRIBUIÇÃO DE PONTOS?

Sim, para o diagrama de Voronoi a velocidade é quase sempre a mesma. Não importa se os pontos estão juntos ou separados, ele faz a mesma quantidade de trabalho bruto (comparar cada ponto com todos os outros). Com isso, seu desempenho acaba sendo previsível.

Já para a triangulação de Delaunay a velocidade muda muito. O desempenho dele depende de quão complexo é adicionar um novo ponto.
> Cenário Rápido (pontos espalhados): Adicionar um novo ponto mexe em poucos triângulos próximos. O trabalho é pequeno e rápido.

> Cenário Lento (pontos em círculo): No pior caso, como ter todos os pontos em um círculo, adicionar um ponto novo no meio pode forçar o algoritmo a refazer quase todos os triângulos de uma vez. Isso cria um gargalo e deixa o processo muito mais lento.

## EXEMPLO DE APLICAÇÃO USANDO DIAGRAMA DE VORONOI

### PLANEJAMENTO DE TRAJETÓRIA

Para um robô navegar em um ambiente com obstáculos, o diagrama pode ser usado para encontrar o "caminho mais seguro", ou seja, a rota que se mantém o mais distante possível de todos os obstáculos.

#### EXEMPLO

Um robô aspirador usa o Diagrama de Voronoi para não bater nos móveis.

Em um ambiente que se deseja que o robô aspire, pode haver diversos obstáculos, sendo eles definidos por apenas um ponto, ou vários (o obstáculo se torna um site). Idealmente o robô não pode simplesmente andar em linha reta até bater em algo. Ele precisa de um mapa que lhe diga onde estão os caminhos seguros para se locomover. Aí que entra o diagrama de Voronoi.

Primeiro, usando seus sensores, o robô aspirador cria um mapa digital da sua sala. Ele identifica a localização e o contorno de todos os objetos como "obstáculos". Então ele é capaz de calcular rotas "seguras".
- Ele trata cada obstáculo (o pé do sofá, a parede, etc...) como um site.
- Ele calcula o Diagrama de Voronoi para todo o mapa. O resultado é uma rede de linhas que se espalha por todo o espaço livre da sala.

Desta forma ele gera um "mapa de estradas seguras". Onde as linhas são exatamente no meio do caminho entre dois obstáculos mais próximos.

Em resumo, o diagrama de Voronoi dá ao robô aspirador a capacidade de analisar e calcular o espaço livre da sua casa, permitindo que ele navegue pelos caminhos mais seguros e lógicos possíveis.
