use ratatui::text::{Line, Span};
use winnow::ascii::line_ending;
use winnow::combinator::{repeat, terminated};
use winnow::{ascii::till_line_ending, PResult, Parser};

pub fn convert(content: &str) -> Vec<Line> {
    let mut content_copy = content;
    match repeat(1.., parse_line).parse_next(&mut content_copy) {
        Ok(lines) => lines,
        Err(_e) => {
            let span = Span::default().content(content.to_owned());
            vec![Line::default().spans(vec![span])]
        }
    }
}

fn parse_line<'s>(input: &mut &'s str) -> PResult<Line<'s>> {
    terminated(till_line_ending, line_ending)
        .parse_next(input)
        .map(|s| Span::default().content(s))
        .map(|span| Line::default().spans(vec![span]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_with_line_ending() -> PResult<()> {
        let mut input = "1. Purpose of Services in Linux:\r\n Plus";
        let result = parse_line(&mut input)?;
        assert_eq!(
            result,
            Line::default().spans(vec![
                Span::default().content("1. Purpose of Services in Linux:")
            ])
        );

        assert_eq!(input, " Plus");
        Ok(())
    }

    #[test]
    fn multiple_lines() {
        let input =
   "1. Purpose of Services in Linux:\r\n
   - Services in Linux are long-running programs or processes that run in the background, providing essential functionality to the system and its users.\r\n
   - They are designed to start automatically when the system boots up, and they continue to run until the system is shut down or the service is manually stopped.\r\n
   - Services are critical for the proper functioning of the operating system and the applications running on it.\r\n";

        let result = super::convert(input);

        assert_eq!(
            result,
            vec![
                Line::default().spans(vec![
                Span::default().content("1. Purpose of Services in Linux:")]),
                Line::default().spans(vec![
                Span::default().content("")]),
                Line::default().spans(vec![
                Span::default().content("   - Services in Linux are long-running programs or processes that run in the background, providing essential functionality to the system and its users.")]),
                Line::default().spans(vec![
                Span::default().content("")]),
                Line::default().spans(vec![
                Span::default().content("   - They are designed to start automatically when the system boots up, and they continue to run until the system is shut down or the service is manually stopped.")]),
                Line::default().spans(vec![
                Span::default().content("")]),
                Line::default().spans(vec![
                Span::default().content("   - Services are critical for the proper functioning of the operating system and the applications running on it.")]),
        ]);
    }
}
