### TODO
- [x] Find out all the functions
- [x] Find out the parameters of the functions
```toml
function_name: <params>
function_name: param1, param2
```
- [ ] Find out all the variables in that function
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
