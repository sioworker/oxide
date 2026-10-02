const VOID: &[&str] = &["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr"];
const RAW: &[&str] = &["script", "style", "textarea", "title"];
const BLK: &[&str] = &["address", "article", "aside", "blockquote", "div", "dl", "fieldset", "footer", "form", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hr", "main", "nav", "ol", "p", "pre", "section", "table", "ul"];

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
	El(El),
	Text(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct El {
	pub tag: String,
	pub attrs: Vec<(String, String)>,
	pub kids: Vec<Node>,
}

impl El {
	pub fn attr(&self, k: &str) -> Option<&str> {
		self.attrs.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str())
	}
}

impl Node {
	pub fn text(&self) -> String {
		match self {
			Node::Text(t) => t.clone(),
			Node::El(e) if matches!(e.tag.as_str(), "script" | "style") => String::new(),
			Node::El(e) => e.kids.iter().map(Node::text).collect(),
		}
	}
}

fn close(st: &mut Vec<El>) {
	let e = st.pop().unwrap();
	st.last_mut().unwrap().kids.push(Node::El(e));
}

fn put(st: &mut [El], t: String) {
	if t.is_empty() {
		return;
	}
	let k = &mut st.last_mut().unwrap().kids;
	match k.last_mut() {
		Some(Node::Text(p)) => p.push_str(&t),
		_ => k.push(Node::Text(t)),
	}
}

fn auto(st: &mut Vec<El>, t: &str) { // implicit end tags
	while st.len() > 1 {
		let x = match st.last().unwrap().tag.as_str() {
			"p" => BLK.contains(&t),
			"li" => t == "li",
			"dt" | "dd" => matches!(t, "dt" | "dd"),
			"option" => matches!(t, "option" | "optgroup"),
			"tr" => t == "tr",
			"td" | "th" => matches!(t, "td" | "th" | "tr"),
			_ => false,
		};
		if !x {
			break;
		}
		close(st);
	}
}

pub fn unesc(s: &str) -> String {
	if !s.contains('&') {
		return s.into();
	}
	let (mut o, mut r) = (String::new(), s);
	while let Some(p) = r.find('&') {
		o += &r[..p];
		r = &r[p..];
		let e = r.find(';').filter(|&e| e < 12);
		let c = e.and_then(|e| match &r[1..e] {
			"amp" => Some('&'),
			"lt" => Some('<'),
			"gt" => Some('>'),
			"quot" => Some('"'),
			"apos" => Some('\''),
			"nbsp" => Some('\u{a0}'),
			n => n.strip_prefix('#').and_then(|n| match n.strip_prefix(['x', 'X']) {
				Some(h) => u32::from_str_radix(h, 16).ok(),
				None => n.parse().ok(),
			}).and_then(char::from_u32),
		});
		match (c, e) {
			(Some(c), Some(e)) => {
				o.push(c);
				r = &r[e + 1..];
			}
			_ => {
				o.push('&');
				r = &r[1..];
			}
		}
	}
	o + r
}

pub fn parse(s: &str) -> Vec<Node> {
	let (b, mut st, mut i) = (s.as_bytes(), vec![El::default()], 0);
	while i < b.len() {
		let r = &s[i..];
		if r.starts_with("<!--") {
			i += r.find("-->").map_or(r.len(), |j| j + 3);
		} else if r.starts_with("<!") || r.starts_with("<?") { // doctype, pi
			i += r.find('>').map_or(r.len(), |j| j + 1);
		} else if r.starts_with("</") && b.get(i + 2).is_some_and(u8::is_ascii_alphabetic) {
			let j = r.find('>');
			let t = r[2..j.unwrap_or(r.len())].trim().to_ascii_lowercase();
			i += j.map_or(r.len(), |j| j + 1);
			if let Some(p) = st.iter().rposition(|e| e.tag == t).filter(|&p| p > 0) {
				while st.len() > p {
					close(&mut st);
				}
			}
		} else if b[i] == b'<' && b.get(i + 1).is_some_and(u8::is_ascii_alphabetic) {
			let mut j = i + 1;
			while j < b.len() && !b[j].is_ascii_whitespace() && !matches!(b[j], b'>' | b'/') {
				j += 1;
			}
			let (tag, mut attrs) = (s[i + 1..j].to_ascii_lowercase(), vec![]);
			loop {
				while j < b.len() && (b[j].is_ascii_whitespace() || b[j] == b'/') {
					j += 1;
				}
				if j >= b.len() || b[j] == b'>' {
					break;
				}
				let k = j;
				j += 1;
				while j < b.len() && !b[j].is_ascii_whitespace() && !matches!(b[j], b'>' | b'=' | b'/') {
					j += 1;
				}
				let n = s[k..j].to_ascii_lowercase();
				while j < b.len() && b[j].is_ascii_whitespace() {
					j += 1;
				}
				let mut v = String::new();
				if j < b.len() && b[j] == b'=' {
					j += 1;
					while j < b.len() && b[j].is_ascii_whitespace() {
						j += 1;
					}
					if j < b.len() && matches!(b[j], b'"' | b'\'') {
						let q = b[j];
						j += 1;
						let k = j;
						while j < b.len() && b[j] != q {
							j += 1;
						}
						v = unesc(&s[k..j]);
						j = (j + 1).min(b.len());
					} else {
						let k = j;
						while j < b.len() && !b[j].is_ascii_whitespace() && b[j] != b'>' {
							j += 1;
						}
						v = unesc(&s[k..j]);
					}
				}
				if !attrs.iter().any(|(a, _)| *a == n) { // first dup wins
					attrs.push((n, v));
				}
			}
			i = (j + 1).min(b.len());
			auto(&mut st, &tag);
			let mut e = El { tag, attrs, kids: vec![] };
			if RAW.contains(&e.tag.as_str()) {
				let r = &s[i..];
				let j = r.to_ascii_lowercase().find(&format!("</{}", e.tag)).unwrap_or(r.len());
				let t = if matches!(e.tag.as_str(), "title" | "textarea") { unesc(&r[..j]) } else { r[..j].into() };
				if !t.is_empty() {
					e.kids.push(Node::Text(t));
				}
				i += r[j..].find('>').map_or(r.len(), |k| j + k + 1);
				st.last_mut().unwrap().kids.push(Node::El(e));
			} else if VOID.contains(&e.tag.as_str()) {
				st.last_mut().unwrap().kids.push(Node::El(e));
			} else {
				st.push(e);
			}
		} else {
			let k = if b[i] == b'<' { i + 1 } else { i }; // stray '<' is text
			let e = s[k..].find('<').map_or(s.len(), |j| k + j);
			put(&mut st, unesc(&s[i..e]));
			i = e;
		}
	}
	while st.len() > 1 {
		close(&mut st);
	}
	st.pop().unwrap().kids
}

#[cfg(test)]
mod tests {
	use super::*;

	fn el(n: &Node) -> &El {
		match n {
			Node::El(e) => e,
			_ => panic!("not el: {n:?}"),
		}
	}

	#[test]
	fn basic() {
		let d = parse("<!DOCTYPE html><html><body><h1 id=t class='a b'>Hello, &amp; World!</h1><!-- x --></body></html>");
		let h = el(&el(&el(&d[0]).kids[0]).kids[0]);
		assert_eq!((h.tag.as_str(), h.attr("id"), h.attr("class")), ("h1", Some("t"), Some("a b")));
		assert_eq!(d[0].text(), "Hello, & World!");
	}

	#[test]
	fn void_raw() {
		let d = parse("<p>a<br>b<img src=x.png/></p><script>if (a<b) x='</p>'</script>");
		assert_eq!(el(&d[0]).kids.len(), 4);
		assert_eq!(el(&el(&d[0]).kids[3]).attr("src"), Some("x.png/"));
		assert_eq!(el(&d[1]).kids[0], Node::Text("if (a<b) x='</p>'".into()));
	}

	#[test]
	fn implicit() {
		let d = parse("<ul><li>a<li>b</ul><p>x<div>y</div><table><tr><td>1<td>2<tr><td>3</table>");
		assert_eq!(el(&d[0]).kids.len(), 2);
		assert_eq!((el(&d[1]).tag.as_str(), el(&d[2]).tag.as_str()), ("p", "div"));
		let t = el(&d[3]);
		assert_eq!((t.kids.len(), el(&t.kids[0]).kids.len()), (2, 2));
	}

	#[test]
	fn junk() {
		let d = parse("a < b &lt;&#65;&#x42;&bogus; </x></b><i>ż");
		assert_eq!(d[0], Node::Text("a < b <AB&bogus; ".into()));
		assert_eq!(d[1].text(), "ż");
		assert_eq!(el(&parse("<a b>")[0]).attr("b"), Some(""));
	}
}
