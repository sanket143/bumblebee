// variables
// - data
// - input

// types
// - data: {
//   id: any,
//   items: Array<{ id: any }>
// }
// - input: any
function giveMeThat(data, config) {
  console.log(data.id);
  console.log(data.items, data["id"]);
  const input = data.items[0].id;

  call(input); // should give an error
}

// variables
// - id

// types
// - id: any
function call({ id }, { config }) {
  console.log(id);
}

const data = {
  id: 1,
  items: [],
};

giveMeThat(data);

