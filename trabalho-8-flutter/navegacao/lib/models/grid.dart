import 'package:navegacao/models/celula.dart';

class Grid {
  int altura;
  int largura;
  late List<List<Celula>> celulas;

  Grid(this.altura, this.largura) {
    celulas = List.generate(altura, generator)
  }
}
