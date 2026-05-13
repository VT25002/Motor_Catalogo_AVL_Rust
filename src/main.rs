// Debug permite imprimir la estructura con {:?} para depuración.
// Clone permite crear copias explícitas de Libro cuando se necesiten.
#[derive(Debug, Clone)]
struct Libro {
    isbn: u32,
    titulo: String,
}

// Option<> Le indica al programa que puede ser un nodo o estar vacío (None).
// Box<Nodo> guarda el nodo en el heap, se usa porque el árbol es una estructura recursiva.
// Rust necesita un tamaño conocido para los tipos almacenados directamente, Box resuelve eso con una referencia administrada.
struct Nodo {
    libro: Libro,
    izquierdo: Option<Box<Nodo>>,
    derecho: Option<Box<Nodo>>,
    altura: i32,
}

impl Nodo {
    // Constructor crea un nodo tipo hoja que tiene altura 1.
    fn nuevo(libro: Libro) -> Self {
        Nodo {
            libro,
            izquierdo: None,
            derecho: None,
            altura: 1,
        }
    }
}

// Devuelve un entero que representa la altura del nodo, si el nodo es None, devuelve 0.
// as_ref() permite leer la altura sin mover ni consumir el Option.
// map_or(0, |n| n.altura) es quien retorna 0 si es None, o la altura del nodo si es Some.
fn obtener_altura(nodo: &Option<Box<Nodo>>) -> i32 {
    nodo.as_ref().map_or(0, |n| n.altura)
}

// Actualiza la altura en base a calculos de los nodos.
// Se usa luego en cada inserción o rotación.
fn actualizar_altura(nodo: &mut Nodo) {
    nodo.altura = 1 + std::cmp::max(
        obtener_altura(&nodo.izquierdo),
        obtener_altura(&nodo.derecho),
    );
}

// Obtiene el balance mediante: Factor de balance = altura_izquierda - altura_derecha.
// > 1 -> subárbol izquierdo demasiado alto (necesita rotación derecha o LR)
// < -1 -> subárbol derecho demasiado alto  (necesita rotación izquierda o RL)
// -1 a 1 -> el árbol está balanceado, no se actúa.
fn obtener_balance(nodo: &Nodo) -> i32 {
    obtener_altura(&nodo.izquierdo) - obtener_altura(&nodo.derecho)
}

// ROTACIÓN DERECHA (caso LL): se aplica cuando el subárbol izquierdo está desbalanceado.
// .take() extrae el hijo izquierdo de `y` (lo mueve fuera del árbol sin dejar referencias colgantes).
// Esto satisface el borrow checker: no podemos tener `y` prestado como dueño mientras también reasignamos sus campos internos.
fn rotar_derecha(mut y: Box<Nodo>) -> Box<Nodo> {
    let mut x = y.izquierdo.take().expect("Hijo izquierdo ausente");
    y.izquierdo = x.derecho.take();
    actualizar_altura(&mut y);
    x.derecho = Some(y);
    actualizar_altura(&mut x);
    x
}

// ROTACIÓN IZQUIERDA (caso RR): se aplica cuando el subárbol derecho está desbalanceado.
fn rotar_izquierda(mut x: Box<Nodo>) -> Box<Nodo> {
    let mut y = x.derecho.take().expect("Hijo derecho ausente");
    x.derecho = y.izquierdo.take();
    actualizar_altura(&mut x);
    y.izquierdo = Some(x);
    actualizar_altura(&mut y);
    y
}

// Inserta un libro en el árbol AVL y retorna la raíz del subárbol.
// Funciona de forma recursiva, ya que baja por el árbol como BST y al subir actualiza alturas y aplica rotaciones si el balance se rompe.
fn insertar(nodo_opt: Option<Box<Nodo>>, libro: Libro) -> Box<Nodo> {
    let mut nodo = match nodo_opt {
        None => return Box::new(Nodo::nuevo(libro)),
        Some(n) => n,
    };

    let isbn_nuevo = libro.isbn;

    if isbn_nuevo < nodo.libro.isbn {
        nodo.izquierdo = Some(insertar(nodo.izquierdo.take(), libro));
    } else if isbn_nuevo > nodo.libro.isbn {
        nodo.derecho = Some(insertar(nodo.derecho.take(), libro));
    } else {
        return nodo;
    }

    actualizar_altura(&mut nodo);
    let balance = obtener_balance(&nodo);

    // Caso LL: desbalance izquierdo, el nuevo nodo está en el subárbol izquierdo-izquierdo.
    if balance > 1 && isbn_nuevo < nodo.izquierdo.as_ref().unwrap().libro.isbn {
        return rotar_derecha(nodo);
    }
    // Caso RR: desbalance derecho, el nuevo nodo está en el subárbol derecho-derecho.
    if balance < -1 && isbn_nuevo > nodo.derecho.as_ref().unwrap().libro.isbn {
        return rotar_izquierda(nodo);
    }
    // Caso LR: desbalance izquierdo, pero el nuevo nodo está en el subárbol izquierdo-derecho.
    if balance > 1 && isbn_nuevo > nodo.izquierdo.as_ref().unwrap().libro.isbn {
        let hijo_izq = nodo.izquierdo.take().unwrap();
        nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        return rotar_derecha(nodo);
    }
    // Caso RL: desbalance derecho, pero el nuevo nodo está en el subárbol derecho-izquierdo.
    if balance < -1 && isbn_nuevo < nodo.derecho.as_ref().unwrap().libro.isbn {
        let hijo_der = nodo.derecho.take().unwrap();
        nodo.derecho = Some(rotar_derecha(hijo_der));
        return rotar_izquierda(nodo);
    }
    nodo
    //Retorna el nodo si no hay desbalance y sin cambios.
}

// Imprime el árbol rotado 90° a la izquierda (derecha = arriba, izquierda = abajo).
// El nivel controla la indentación visual para mostrar la jerarquía.
fn imprimir(nodo: &Option<Box<Nodo>>, nivel: usize) {
    if let Some(n) = nodo {
        imprimir(&n.derecho, nivel + 1);
        println!(
            "{:indent$}[ISBN: {}] {}",
            "",
            n.libro.isbn,
            n.libro.titulo,
            indent = nivel * 4
        );
        imprimir(&n.izquierdo, nivel + 1);
    }
}

// Busca un libro por ISBN recorriendo el árbol (sin copiar datos).
// Retorna Some(&Libro) si lo encuentra, None si no existe.
// Al usar &Libro evitamos clonar: solo prestamos una referencia al dato original.
fn buscar(nodo: &Option<Box<Nodo>>, isbn: u32) -> Option<&Libro> {
    match nodo {
        None => None,
        Some(n) => {
            if isbn == n.libro.isbn {
                Some(&n.libro)
            } else if isbn < n.libro.isbn {
                buscar(&n.izquierdo, isbn)
            } else {
                buscar(&n.derecho, isbn)
            }
        }
    }
}

// Aplica las rotaciones necesarias según el factor de balance del nodo.
// Se reutiliza tanto en insertar como en eliminar para mantener el árbol balanceado.
fn balancear(mut nodo: Box<Nodo>) -> Box<Nodo> {
    actualizar_altura(&mut nodo);
    let balance = obtener_balance(&nodo);

    // Caso LL: subárbol izquierdo-izquierdo
    if balance > 1 && obtener_balance(nodo.izquierdo.as_ref().unwrap()) >= 0 {
        return rotar_derecha(nodo);
    }
    // Caso LR: subárbol izquierdo-derecho
    if balance > 1 && obtener_balance(nodo.izquierdo.as_ref().unwrap()) < 0 {
        let hijo_izq = nodo.izquierdo.take().unwrap();
        nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        return rotar_derecha(nodo);
    }
    // Caso RR: subárbol derecho-derecho
    if balance < -1 && obtener_balance(nodo.derecho.as_ref().unwrap()) <= 0 {
        return rotar_izquierda(nodo);
    }
    // Caso RL: subárbol derecho-izquierdo
    if balance < -1 && obtener_balance(nodo.derecho.as_ref().unwrap()) > 0 {
        let hijo_der = nodo.derecho.take().unwrap();
        nodo.derecho = Some(rotar_derecha(hijo_der));
        return rotar_izquierda(nodo);
    }

    nodo
}

// Elimina el nodo con el ISBN dado y rebalancea el árbol según 3 casos:
// Caso 1 - Nodo hoja (sin hijos): se elimina directamente retornando None.
// Caso 2 - Nodo con un hijo: se reemplaza por su único hijo.
// Caso 3 - Nodo con dos hijos: se reemplaza con el sucesor in-orden (mínimo del subárbol derecho) y se elimina ese sucesor.
fn eliminar(nodo_opt: Option<Box<Nodo>>, isbn: u32) -> Option<Box<Nodo>> {
    let mut nodo = match nodo_opt {
        None => return None, // ISBN no existe en el árbol
        Some(n) => n,
    };

    if isbn < nodo.libro.isbn {
        nodo.izquierdo = eliminar(nodo.izquierdo.take(), isbn);
    } else if isbn > nodo.libro.isbn {
        nodo.derecho = eliminar(nodo.derecho.take(), isbn);
    } else {
        // Si el nodo se encuentra determinar que caso aplica:

        // Caso 1 y 2: sin hijo izquierdo -> subir el derecho (puede ser None)
        if nodo.izquierdo.is_none() {
            return nodo.derecho;
        }

        // Caso 2: sin hijo derecho -> subir el izquierdo
        if nodo.derecho.is_none() {
            return nodo.izquierdo;
        }

        // Caso 3: dos hijos -> buscar sucesor in-orden (mínimo del subárbol derecho)
        let sucesor_libro = {
            let mut actual = nodo.derecho.as_ref().unwrap();
            while let Some(ref izq) = actual.izquierdo {
                actual = izq;
            }
            actual.libro.clone() // Se necesita el dato antes de eliminar el nodo
        };

        // Eliminar el sucesor del subárbol derecho y colocar su dato en el nodo actual
        nodo.derecho = eliminar(nodo.derecho.take(), sucesor_libro.isbn);
        nodo.libro = sucesor_libro;
    }

    // Rebalancear al subir en la recursión
    Some(balancear(nodo))
}

fn main() {
    let mut raiz: Option<Box<Nodo>> = None;
    let datos = vec![
        (10, "El Quijote"),
        (20, "1984"),
        (30, "Hamlet"),
        (5, "Fahrenheit 451"),
        (2, "La Odisea"),
        (25, "El Principito"),
    ];

    println!("--- Sistema de Inventario de Librería (AVL) ---");
    for (isbn, titulo) in datos {
        let libro = Libro {
            isbn,
            titulo: titulo.to_string(),
        };
        raiz = Some(insertar(raiz.take(), libro));
    }

    imprimir(&raiz, 0);

    // ============================================================
    // FASE 1 - Prueba de Escritorio
    // Inserción: [10, 20, 30, 5, 2, 25]
    //
    // Árbol final:
    //
    //       20
    //      /  \
    //     5    30
    //    / \   /
    //   2  10 25
    //
    // Rotaciones ocurridas:
    //   1. Insertar 30 → Nodo 10 queda con balance -2 (caso RR)
    //                    Se aplica rotar_izquierda(10)
    //   2. Insertar  2 → Nodo 10 queda con balance +2 (caso LL)
    //                    Se aplica rotar_derecha(10)
    //
    // Análisis de .take() en Rust
    // En Rust cada valor tiene un solo dueño, mover un campo directamente fuera
    // de una Box viola esta regla porque la Box aún reclama ese dato como propio.
    // .take() resuelve esto al extraer el contenido del Option reemplazandolo con None y
    // transfiriendo el ownership del hijo de forma limpia sin dejar referencias colgantes.
    // Sin esto, reorganizar subárboles en una rotación generaría errores del borrow checker
    // ya que estaríamos intentando mover y usar el mismo nodo desde dos lugares a la vez.
    // ============================================================

    // --- ESPACIO PARA TUS PRUEBAS ---

    // FASE 2 - Búsqueda por ISBN
    println!("\n--- Búsqueda en el inventario ---");

    println!("\nPrueba con ISBN: 25");
    match buscar(&raiz, 25) {
        Some(libro) => println!(
            "Libro encontrado con datos ISBN: {}, título: {}",
            libro.isbn, libro.titulo
        ),
        None => println!("Libro no encontrado."),
    }

    println!("\nPrueba con ISBN: 99");
    match buscar(&raiz, 99) {
        Some(libro) => println!(
            "Libro encontrado con datos ISBN: {}, título: {}",
            libro.isbn, libro.titulo
        ),
        None => println!("Libro no encontrado."),
    }

    // FASE 3 - Eliminación
    println!("\n--- Eliminación según diferentes casos ---");

    // Caso 1: eliminar nodo hoja (ISBN 2, sin hijos)
    println!("\nEliminación Caso 1: nodo hoja (ISBN 2)");
    raiz = eliminar(raiz.take(), 2);
    imprimir(&raiz, 0);

    // Caso 2: eliminar nodo con un hijo (ISBN 30, solo tiene hijo izquierdo: 25)
    println!("\nEliminación Caso 2: un hijo (ISBN 30)");
    raiz = eliminar(raiz.take(), 30);
    imprimir(&raiz, 0);

    // Caso 3: eliminar nodo con dos hijos (ISBN 5, tiene izquierdo y derecho)
    println!("\nEliminación Caso 3: dos hijos (ISBN 5)");
    raiz = eliminar(raiz.take(), 5);
    imprimir(&raiz, 0);

    // Caso borde: eliminar ISBN inexistente
    println!("\nEliminación de ISBN inexistente (ISBN 99)");
    raiz = eliminar(raiz.take(), 99);
    imprimir(&raiz, 0);
}
