use tombi_config::{
    ArrayBracketSpaceWidth, ArrayCommaSpaceWidth, FormatOptions, IndentStyle, IndentWidth, LineEnding, LineWidth,
    format::FormatRules,
};

pub fn create_format_options(column_width: usize, indent: usize) -> FormatOptions {
    FormatOptions {
        rules: Some(FormatRules {
            line_width: Some(LineWidth::try_from(column_width as u8).unwrap_or_default()),
            indent_style: Some(IndentStyle::Space),
            indent_width: Some(IndentWidth::from(indent as u8)),
            line_ending: Some(LineEnding::Lf),
            array_bracket_space_width: Some(ArrayBracketSpaceWidth::from(1)),
            array_comma_space_width: Some(ArrayCommaSpaceWidth::from(1)),
            ..Default::default()
        }),
    }
}
