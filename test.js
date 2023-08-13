function giveMeThat(input) {
  console.log(input.id);
  console.log(input.items, input["id"]);

  function helloWorld({ id }) {
    console.log(id.value);
  }
  {
    const scope = 1;
  }
}

function testFunction({ id }, { user: { token } }) {
  console.log(id);
}

const data = {
  id: 1,
  items: [],
};

giveMeThat(data);
testFunction(data);

{
  const scope2 = 2;
}
