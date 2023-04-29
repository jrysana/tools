use rome_analyze::{context::RuleContext, declare_rule, ActionCategory, Ast, Rule, RuleDiagnostic};
use rome_console::markup;
use rome_diagnostics::Applicability;
use rome_js_factory::make;
use rome_js_syntax::{AnyJsxChild, JsxText, TriviaPieceKind, T};
use rome_rowan::{AstNode, BatchMutationExt};

use crate::JsRuleAction;
declare_rule! {
    /// Prevent comments from being inserted as text nodes
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// const a3 = <div>// comment</div>;
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// const a4 = <div>/* comment */</div>;
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// const a5 = <div>/** comment */</div>;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```js
    /// const a = <div>{/* comment */}</div>;
    /// const a1 = <div>{/** comment */}</div>;
    /// const a2 = <div className={"cls" /* comment */}></div>;
    /// ```
    ///
    /// ## Corresponding ESLint rules
    ///
    /// - [react/jsx-no-comment-textnodes(https://github.com/jsx-eslint/eslint-plugin-react/blob/master/docs/rules/jsx-no-comment-textnodes.md)
    ///
    pub(crate) NoCommentText {
        version: "0.7.0",
        name: "noCommentText",
        recommended: true,
    }
}

impl Rule for NoCommentText {
    type Query = Ast<JsxText>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let n = ctx.query();
        let jsx_value = n.text();
        let is_single_line_comment = jsx_value.starts_with("//");
        let is_multi_line_comment = jsx_value.starts_with("/*") && jsx_value.ends_with("*/");
        if is_single_line_comment || is_multi_line_comment {
            Some(())
        } else {
            None
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, _: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();

        Some(RuleDiagnostic::new(
            rule_category!(),
            node.range(),
            markup! {
                "Wrap "<Emphasis>"comments"</Emphasis>" inside children within "<Emphasis>"braces"</Emphasis>"."
            },
        ))
    }

    fn action(ctx: &RuleContext<Self>, _: &Self::State) -> Option<JsRuleAction> {
        let node = ctx.query();
        let mut mutation = ctx.root().begin();

        let normalized_comment = format!(
            "/*{}*/",
            node.text()
                .trim_start_matches("/**")
                .trim_start_matches("//")
                .trim_start_matches("/*")
                .trim_end_matches("*/")
        );

        mutation.replace_node(
            AnyJsxChild::JsxText(node.clone()),
            AnyJsxChild::JsxExpressionChild(
                make::jsx_expression_child(
                    make::token(T!['{']).with_trailing_trivia(
                        [(
                            TriviaPieceKind::MultiLineComment,
                            normalized_comment.as_str(),
                        )]
                        .into_iter(),
                    ),
                    make::token(T!['}']),
                )
                .build(),
            ),
        );

        Some(JsRuleAction {
            category: ActionCategory::QuickFix,
            applicability: Applicability::MaybeIncorrect,
            message: markup! { "Wrap the comments with braces" }.to_owned(),
            mutation,
        })
    }
}
