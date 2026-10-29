# Basic Markdown to HTML

Before/after example: headings, links, inline HTML, lists, a table, and a fenced
code block, converted to their HTML equivalents.

````text
    ```markdown
    # Level 1
    ## Level 2

    One sentence with a [link](https://example.com), and a HTML snippet like `<p>paragraph tag</p>`.

    - `ul` list item 1
    - `ul` list item 2

    1. `ol` list item 1
    2. `ol` list item 1

    | Table Item | Description |
    | One | One is the spelling of the number `1`. |
    | Two | Two is the spelling of the number `2`. |

    ```js
    var one = 1;
    var two = 2;

    function simpleMath(x, y) {
     return x + y;
    }
    console.log(simpleMath(one, two));
    ```
    ```

    ```html
    <h1>Level 1</h1>
    <h2>Level 2</h2>

    <p>One sentence with a <a href="https://example.com">link</a>, and a HTML snippet like <code>&lt;p&gt;paragraph tag&lt;/p&gt;</code>.</p>

    <ul>
     <li>`ul` list item 1</li>
     <li>`ul` list item 2</li>
    </ul>

    <ol>
     <li>`ol` list item 1</li>
     <li>`ol` list item 2</li>
    </ol>

    <table>
     <thead>
      <tr>
       <th>Table Item</th>
       <th>Description</th>
      </tr>
     </thead>
     <tbody>
      <tr>
       <td>One</td>
       <td>One is the spelling of the number `1`.</td>
      </tr>
      <tr>
       <td>Two</td>
       <td>Two is the spelling of the number `2`.</td>
      </tr>
     </tbody>
    </table>

    <pre>
     <code>var one = 1;
     var two = 2;

     function simpleMath(x, y) {
      return x + y;
     }
     console.log(simpleMath(one, two));</code>
    </pre>
    ```
````
