# Lyron
A... programming language...

***NOTE:** It is a work in progress..*

# Syntax
* Note: The syntax might (and most probably will) undergo changes.
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
        let a:i32 = (-b + 5) - 10 / -(5 - -2)
        ```
* Comments
    * Comments start with `#` and continue until the end of the line
        ```
        # this is a comment
        ```

* Programs
    * A program consists of just top-level functions, classes, and expressions.

