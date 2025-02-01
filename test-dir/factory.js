const factory = {
  test: (data) => {
    console.log(data.id);
  },
};

export const call = () => {
  factory.test({ id: 1 });
};
