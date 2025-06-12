export const idlFactory = ({ IDL }) => {
  return IDL.Service({ 'fetch_data' : IDL.Func([], [IDL.Text], ['query']) });
};
export const init = ({ IDL }) => { return []; };
