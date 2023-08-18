function giveMeThat(data, config) {
  console.log(data.id);
  console.log(data.items, data["id"]);
  const input = data.items[0].id;

  call(input);
}

function call({ token }, { config }, ...args) {
  console.log(toast);
}

giveMeThat(data);

