import dts from 'rollup-plugin-dts';

export default {
  input: 'src/index.ts',
  output: [
    {
      dir: 'lib',
      format: 'es',
    },
  ],
  plugins: [
    dts({
      tsconfig: './tsconfig.json',
    }),
  ],
};
