Queso is a PoC for derived type checking for normal JS projects

### TODO
- [x] Find out all the functions
- [x] Find out the parameters of the functions
```
function_name: <params>
function_name: param1, param2
```
- [ ] Identify the leaf property of the input params, that'll be one of the var in the function scope
- [ ] Find out all the variables in that function
Only need to track relevant variables, i.e. which are in use in the the body
```js
// `config` and `args` are not relevant
function call({ token }, { config }, ...args) {
  // relevant if is used in a `callExpression`
  console.log(token);

  // relevant if a property is being accessed;
  token.id;
}
```
- [ ] Figure out which of the variable is a parameter to that function
- [ ] Derive type for each variables
- [ ] Derive return type of the function based on the returning variable or structure
- [ ] Consider the data returned by a function also when deriving the type
```js
function call(input){
  return {
    id: input.id,
    items: getItemsById(input.id) // assume the result is an array of objects
  }
}


/**
 * `data` Should have type
 *
 * @param {Object} data.id // technically `Object` is `any`
 * @param {Array<Object>} data.items
 */
const data = call({ id: 2 });
```
- [ ] Handle Rest elements in the function


### Unhandled TODO
- [ ] Following will not throw an error
```js
function call(input){
  input.id;
  input.items;
}

const data = {
  id: "1"
};

call(data);
data.items = [];
```

### NOTES
- Variables extracted from the parameters != input signature of the function
