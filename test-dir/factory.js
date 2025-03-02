const factory = {
  test: (data) => {
    var test_symbol = 0;
    console.log(data.id);
  },
};

export const call = () => {
  factory.test({ id: 1 });
};

const test_symbol_root = 0;
