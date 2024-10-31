use common::taplo::rowan::SyntaxNode;
use common::taplo::syntax::Lang;

use common::table::Tables;

pub fn reorder_tables(root_ast: &SyntaxNode<Lang>, tables: &Tables) {
    tables.reorder(root_ast, &["", "env_run_base", "env"]);
}
