Feature: Variables
  Scenario: Define a global variable
    Given a file named "main.sl" with:
    """
    y : Number
    y = 42

    main : Number -> Number
    main x = y
    """
    And I successfully run `builder`
    When I run `sh -c ./a.out`
    Then stdout from "sh -c ./a.out" should contain exactly "42"
    And the exit status should be 0

  Scenario: Use let-values expression
    Given a file named "main.sl" with:
    """
    main : Number -> Number
    main x = (
      let
        y : Number
        y = x
      in
        y
    )
    """
    And I successfully run `builder`
    When I run `sh -c ./a.out`
    Then stdout from "sh -c ./a.out" should contain exactly "42"
    And the exit status should be 0

  Scenario: Use untyped let-values expression
    Given a file named "main.sl" with:
    """
    main : Number -> Number
    main x = (
      let
        y = x
      in
        y
    )
    """
    And I successfully run `builder`
    When I run `sh -c ./a.out`
    Then stdout from "sh -c ./a.out" should contain exactly "42"
    And the exit status should be 0

  Scenario: Use nested let-values expression
    Given a file named "main.sl" with:
    """
    main : Number -> Number
    main x = (
      let
        y = (
          let
            z = x
          in
            z
        )
      in
        y
    )
    """
    And I successfully run `builder`
    When I run `sh -c ./a.out`
    Then stdout from "sh -c ./a.out" should contain exactly "42"
    And the exit status should be 0

  Scenario: Use let-functions expression
    Given a file named "main.sl" with:
    """
    main : Number -> Number
    main x = (
      let
        f : Number -> Number
        f y = y
      in
        f x
    )
    """
    And I successfully run `builder`
    When I run `sh -c ./a.out`
    Then stdout from "sh -c ./a.out" should contain exactly "42"
    And the exit status should be 0

  Scenario: Use untyped let-functions expression
    Given a file named "main.sl" with:
    """
    main : Number -> Number
    main x = (
      let
        f y = y
      in
        f x
    )
    """
    And I successfully run `builder`
    When I run `sh -c ./a.out`
    Then stdout from "sh -c ./a.out" should contain exactly "42"
    And the exit status should be 0

  Scenario: Define multiple functions in a let-functions expression
    Given a file named "main.sl" with:
    """
    main : Number -> Number
    main x = (
      let
        f y = y
        g z = f z
      in
        g x
    )
    """
    And I successfully run `builder`
    When I run `sh -c ./a.out`
    Then stdout from "sh -c ./a.out" should contain exactly "42"
    And the exit status should be 0
