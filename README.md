# Lyron
A simple dynamically typed (with type annotations), lightweight programming language designed to be simple and clear.
Your feedback and contributions are welcome!


## Installation

1. Clone the repo
   ```bash
   git clone https://github.com/re-masashi/lyron.git
   cd lyron
   ```
2. Build Lyron
    ```bash
    cargo build --release
    ```
    or, if you want to have GxHash (a faster hashmap implementation) enabled.
    ```bash
    RUSTFLAGS="-C target-cpu=native" cargo build --release --features gxhash
    ```
---

## Quickstart

Create `hello.ly`:

```lyron
# hello.ly
print("Hello, Lyron!")
```

Run it:

```bash
lyron hello.ly
# → Hello, Lyron!
```
---

# Syntax
* Functions:
    * Defining a function
        ```
        def sum(a:i32, b:i32) -> i32
            a+b
        end 
        ```
    * Calling a function
        ```
        sum(a, b)
        ```
* Class
    * A class in lyron is just a collection of functions and attributes
    * Constructors should have the same name as the class. Eg:
        ```
        class Animal{
            def Animal(self: Self, age: i32)->Animal do
                print("i'm an animal ")
                print("i'm "+age+" years old.")
                self = setattr(self, "age", age) # `self.age = age` is invalid!
                self
            end

            def sound(self: Self) -> Animal
                print("i make a sound")
        }
        ```

* Variables:
    * Declaration:
        ```
        let x: i32    
        ```
    * Assignment:
        ```
        x = 42 
        ```
    Note: Both need to be done separately.

* Operations
    * Available operations `=`, `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `+=`, `-=`, `*=`, `/=`
        ```
        let a: i32 = (-b + 5) - 10 / -(5 - -2)
        ```
* Comments
    * Comments start with `#` and continue until the end of the line
        ```
        # this is a comment
        ```

* Programs
    * A program consists of just top-level functions, classes, and expressions.

## Contributing

1. Fork the repo  
2. Create a branch: `git checkout -b feature/YourFeature`  
3. Make your changes & add tests  
4. Submit a PR against `main`

## Roadmap

- [x] Core parser improvements (better errors, etc) 
- [ ] ?Standard library: collections, I/O
- [x] Module system
- [ ] Package Manager
- [ ] Performance Improvements (make it 4x faster than now, 9th April 2025)

*Note: ? indicates half-done stuff*

## License

MIT © Masashi
