#[macro_export]
macro_rules! cyk {
    (Start($start:literal);NonTerminal[$($non_terminal:literal),+ $(,)?];Terminal[$($terminal:literal),+ $(,)?];Rules[$($left:literal => [$([$($right:literal),+ $(,)?]),+ $(,)?]),+ $(,)?]) => {
        {
            let start_terminal = $start;
            let non_terminals = [$($non_terminal),+];
            let terminals = [$($terminal),+];
            let rules = [$(($left, [$([$($right),+]),+])),+];

            // panic!("adsfadsf");

            rules
        }
    };
}