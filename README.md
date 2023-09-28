# Lyron
A...programming language...

***NOTE:** It is a work in progress..*

# Syntax
* Note: The syntax is subject to change.
* Functions:
    * Regular function
        ```
        def sum(a:i32, b:i32) -> i32 {
            return a + b;
        }
        ```
    * External functions
        ```
        extern sin(num:f32);
        ```
    * Calling a function
        ```
        sum(a, b);
        ```
* Classes:
    ```
    class Name{
        // functions
    }
    
    ```

* Variables:
    * Declaration:
    ```
        let x: i32;    
    ```
    * Assignment:
    ```
        x = 42;    
    ```
    Note: Both need to be done separately.

* Operations
    * Available operations `=`, `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `+=`, `-=`, `*=`, `/=`
        ```
        let a:i32 = (-b + 5) - 10 / -(5 - -2);
        ```
* Comments
    * Comments start with `#` and go until the end of the line

* Programs
    * A program consists of just top-level functions, `extern` definitions, and expressions.

