pub mod to_external {
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::convert::TryFrom;

    use lol_html::rewrite_str;
    use ruma::identifiers::{RoomAliasId, UserId};

    pub use lol_html::{
        html_content::{ContentType, Element},
        ElementContentHandlers, Settings,
    };

    type UserMapper<'a> = &'a dyn Fn(UserId, &Info) -> Option<String>;
    type RoomMapper<'a> = &'a dyn Fn(RoomAliasId, &Info) -> Option<String>;
    type ElementClosure<'a> = &'a dyn Fn(&mut Element<'_, '_>, &Info);

    pub struct Info<'a> {
        user_mapper: UserMapper<'a>,
        room_mapper: RoomMapper<'a>,
        element_handlers: HashMap<String, ElementClosure<'a>>,
    }

    pub fn generate_user_mapper_from_hashmap(
        map: HashMap<UserId, String>,
    ) -> impl Fn(UserId, &Info) -> Option<String> {
        move |user_id: UserId, _: &Info| -> Option<String> { map.get(&user_id).cloned() }
    }

    pub fn generate_room_mapper_from_hashmap(
        map: HashMap<RoomAliasId, String>,
    ) -> impl Fn(RoomAliasId, &Info) -> Option<String> {
        move |room_id: RoomAliasId, _: &Info| -> Option<String> { map.get(&room_id).cloned() }
    }

    impl<'a> Info<'a> {
        pub fn new() -> Self {
            Self {
                user_mapper: &|_: UserId, _: &Info| None,
                room_mapper: &|_: RoomAliasId, _: &Info| None,
                element_handlers: HashMap::new(),
            }
        }

        pub fn user_mapper(&mut self, f: &'a dyn Fn(UserId, &Info) -> Option<String>) -> &mut Self {
            self.user_mapper = f;
            self
        }

        pub fn room_mapper(
            &mut self,
            f: &'a dyn Fn(RoomAliasId, &Info) -> Option<String>,
        ) -> &mut Self {
            self.room_mapper = f;
            self
        }

        pub fn add_element_handler(&mut self, element: String, f: ElementClosure<'a>) -> &mut Self {
            self.element_handlers.insert(element, f);
            self
        }
    }

    impl Default for Info<'_> {
        fn default() -> Self {
            Info::new()
        }
    }

    fn stringify_a_tag(el: &mut Element, info: &Info) {
        let normal = |el: &mut Element, url: Option<String>| match info.element_handlers.get("a") {
            Some(f) => {
                f(el, info);
            }
            None => {
                el.remove_and_keep_content();

                if let Some(url) = url {
                    el.prepend("[", ContentType::Html);
                    el.append(&format!("]({})", url), ContentType::Html);
                }
            }
        };

        let href = match el.get_attribute("href") {
            Some(href) => href,
            _ => return normal(el, None),
        };

        let mentioned = match href.strip_prefix("https://matrix.to/#/") {
            None => return normal(el, Some(href)),
            Some(suffix) => suffix,
        };

        let s = match mentioned.chars().next() {
            Some('@') => {
                let mentioned = UserId::try_from(mentioned).unwrap();
                (info.user_mapper)(mentioned, info)
            }
            Some('#') => {
                let room = RoomAliasId::try_from(mentioned).unwrap();
                (info.room_mapper)(room, info)
            }
            _ => None,
        };

        if let Some(s) = s {
            el.replace(&s, ContentType::Html)
        } else {
            normal(el, Some(href))
        }
    }

    pub fn convert(s: &str, info: &Info) -> Result<String, &'static str> {
        let mut settings = Settings::default();

        settings.element_content_handlers = vec![
            ((
                Cow::Owned("body".parse().unwrap()),
                ElementContentHandlers::default().comments(|c| {
                    c.remove();
                    Ok(())
                }),
            )),
            ((
                Cow::Owned("a".parse().unwrap()),
                ElementContentHandlers::default().element(|e| {
                    stringify_a_tag(e, info);
                    Ok(())
                }),
            )),
            ((
                Cow::Owned("*".parse().unwrap()),
                ElementContentHandlers::default().element(|e| {
                    let tag = e.tag_name();
                    if let Some(handler) = info.element_handlers.get(&tag) {
                        handler(e, info);
                    } else {
                        e.remove_and_keep_content();
                    }
                    Ok(())
                }),
            )),
        ];

        Ok(rewrite_str(s, settings).unwrap())
    }

    #[cfg(test)]
    mod tests {
        use std::collections::HashMap;
        use std::convert::TryFrom;

        use lol_html::html_content::ContentType;
        use ruma::identifiers::{user_id, RoomAliasId, UserId};

        use crate::convert::to_external::{convert, Element, Info};

        #[test]
        fn test_stripping() {
            let info = Info::new();

            let before = "<b>kaas</b>".to_string();
            let after = convert(&before, &info).unwrap();
            assert_eq!(after, "kaas");
        }

        #[test]
        fn test_anchor() {
            let mut user_mapping = HashMap::new();
            user_mapping.insert(user_id!("@tomsg_tom:lieuwe.xyz"), "tom".to_string());

            let mut info = Info::new();
            let f = move |user_id: UserId, _: &Info| user_mapping.get(&user_id).cloned();
            info.user_mapper(&f);

            let before =
                "<a href=\"https://matrix.to/#/@tomsg_tom:lieuwe.xyz\">tom (tomsg)</a>".to_string();

            let after = convert(&before, &info).unwrap();
            assert_eq!(after, "tom");
        }

        #[test]
        fn test_anchor_room() {
            let mut room_mapping = HashMap::new();
            room_mapping.insert(
                RoomAliasId::try_from("#tomsg:lieuwe.xyz").unwrap(),
                "tomsg".to_string(),
            );

            let mut info = Info::new();
            let f = move |room_id: RoomAliasId, _: &Info| room_mapping.get(&room_id).cloned();
            info.room_mapper(&f);

            let before = "<a href=\"https://matrix.to/#/#tomsg:lieuwe.xyz\">tomsg</a>".to_string();

            let after = convert(&before, &info).unwrap();
            assert_eq!(after, "tomsg");
        }

        #[test]
        fn test_complex() {
            let mut user_mapping = HashMap::new();
            user_mapping.insert(user_id!("@tomsg_tom:lieuwe.xyz"), "tom".to_string());
            user_mapping.insert(user_id!("@lieuwe:lieuwe.xyz"), "lieuwe".to_string());

            let mut info = Info::new();
            let f = move |user_id: UserId, _: &Info| user_mapping.get(&user_id).cloned();
            info.user_mapper(&f);

            let before =
            "<a href=\"https://matrix.to/#/@tomsg_tom:lieuwe.xyz\">tom (tomsg)</a>: How're you doing, greetings <a href=\"https://matrix.to/#/@lieuwe:lieuwe.xyz\">henk</a>. Btw, here is a cool link <a href=\"google.nl\">bing</a>".to_string();

            let after = convert(&before, &info).unwrap();
            assert_eq!(
            after,
            "tom: How're you doing, greetings lieuwe. Btw, here is a cool link [bing](google.nl)"
        );
        }

        #[test]
        fn test_element_handlers() {
            let mut info = Info::new();
            info.add_element_handler("a".to_string(), &|el: &mut Element<'_, '_>, _: &Info| {
                el.replace("test", ContentType::Html);
            });

            let before = "<a href=\"google.nl\">this will be gone</a>";

            let after = convert(&before, &info).unwrap();
            assert_eq!(after, "test");
        }

        #[test]
        fn test_complex2() {
            let before = "<mx-reply><blockquote><a href=\"https://matrix.to/#/!opVyAOHWsarCVcEQkE:lieuwe.xyz/$wjpDcX-sy3dLophlXRfL0pyE4yotZ5XK8v1DF_VMpoU?via=lieuwe.xyz\">In reply to</a> <a href=\"https://matrix.to/#/@tomsg_tom:lieuwe.xyz\">@tomsg_tom:lieuwe.xyz</a><br>⛄️</blockquote></mx-reply>Hallo <a href=\"https://matrix.to/#/@tomsg_tom:lieuwe.xyz\">tom (tomsg)</a> dit is een test <em>kaas</em> <strong>ham</strong> <a href=\"http://tomsmeding.com/f/kaas.png\">coole site</a>";
            let after = "Hallo tom dit is een test *kaas* **ham** [coole site](http://tomsmeding.com/f/kaas.png)";

            let mut info = Info::new();

            info.add_element_handler(
                "mx-reply".to_string(),
                &|el: &mut Element<'_, '_>, _: &Info| el.remove(),
            );
            info.add_element_handler("em".to_string(), &|el: &mut Element<'_, '_>, _: &Info| {
                el.prepend("*", lol_html::html_content::ContentType::Html);
                el.remove_and_keep_content();
                el.append("*", lol_html::html_content::ContentType::Html);
            });
            info.add_element_handler(
                "strong".to_string(),
                &|el: &mut Element<'_, '_>, _: &Info| {
                    el.prepend("**", lol_html::html_content::ContentType::Html);
                    el.remove_and_keep_content();
                    el.append("**", lol_html::html_content::ContentType::Html);
                },
            );

            let mut user_mapping: HashMap<UserId, String> = HashMap::new();
            user_mapping.insert(user_id!("@tomsg_tom:lieuwe.xyz"), "tom".to_string());

            let f = move |user_id: UserId, _: &Info| user_mapping.get(&user_id).cloned();
            info.user_mapper(&f);

            assert_eq!(after, convert(&before, &info).unwrap());
        }
    }
}

pub mod to_matrix {
    use std::collections::HashMap;

    use crate::matrix::MatrixToItem;

    // HACK
    use regex::escape;

    use pcre2::bytes::Regex;

    pub struct Info<'a> {
        pub map: HashMap<String, MatrixToItem<'a>>,
    }

    pub struct BuiltRegex(Regex);

    pub fn build_regex(info: &Info) -> BuiltRegex {
        let mut regex_string = r"(?<=^|\W)(".to_string();
        for (i, (key, _)) in info.map.iter().enumerate() {
            if i > 0 {
                regex_string += "|";
            }

            regex_string += &escape(key);
        }
        regex_string += r")(?=$|\W)";

        let regex = Regex::new(&regex_string).unwrap();
        BuiltRegex(regex)
    }

    pub fn convert(regex: BuiltRegex, mut s: String, info: &Info) -> String {
        let s_cloned = s.clone();
        // find names that are in the map, and replace them with an url.
        let captures = regex.0.captures_iter(s_cloned.as_bytes());

        let mut delta = 0i64;
        for cap in captures {
            let m = cap.unwrap().get(1).unwrap();
            // safety: we know the input bytes to the regex is well-formed utf-8, the regex only
            // works on utf-8 runes.  Therefore, the capture group also must only contain
            // well-formed utf-8 bytes.
            let name = unsafe { std::str::from_utf8_unchecked(m.as_bytes()) };
            let to = info.map.get(name).unwrap();
            let to = to.to_url_string();
            let to = format!("<a href=\"{}\">{}</a>", to, name);

            let start = (m.start() as i64 + delta) as usize;
            let end = (m.end() as i64 + delta) as usize;

            s.replace_range(start..end, &to);

            delta += (to.len() as i64) - ((end - start) as i64);
        }

        s
    }

    #[cfg(test)]
    mod tests {
        use std::collections::HashMap;

        use ruma::identifiers::user_id;

        use crate::convert::to_matrix::{build_regex, convert, Info};
        use crate::MatrixToItem;

        #[test]
        fn test_mapping() {
            let before = "hello tom";
            let after = "hello <a href=\"https://matrix.to/#/@tomsg_tom:lieuwe.xyz\">tom</a>";

            let mut map = HashMap::new();
            let sed = user_id!("@tomsg_tom:lieuwe.xyz");
            map.insert("tom".to_string(), MatrixToItem::User(&sed));

            let info = Info { map };
            let regex = build_regex(&info);

            assert_eq!(after, convert(regex, before.to_string(), &info));
        }

        #[test]
        fn test_mapping2() {
            let before = "hello sed[m]";
            let after = "hello <a href=\"https://matrix.to/#/@sed:t2bot.io\">sed[m]</a>";

            let mut map = HashMap::new();
            let sed = user_id!("@sed:t2bot.io");
            map.insert("sed[m]".to_string(), MatrixToItem::User(&sed));

            let info = Info { map };
            let regex = build_regex(&info);

            assert_eq!(after, convert(regex, before.to_string(), &info));
        }

        #[test]
        fn test_mapping_double() {
            let before = "hello sed[m] voyager[m]";
            let after = "hello <a href=\"https://matrix.to/#/@sed:t2bot.io\">sed[m]</a> <a href=\"https://matrix.to/#/@voyager:t2bot.io\">voyager[m]</a>";

            let mut map = HashMap::new();
            let sed = user_id!("@sed:t2bot.io");
            map.insert("sed[m]".to_string(), MatrixToItem::User(&sed));
            let voyager = user_id!("@voyager:t2bot.io");
            map.insert("voyager[m]".to_string(), MatrixToItem::User(&voyager));

            let info = Info { map };
            let regex = build_regex(&info);

            assert_eq!(after, convert(regex, before.to_string(), &info));
        }
    }
}

/*
use scraper::{Html, NodeMut, Selector};

fn clean_matrix_tree(node: &mut NodeMut) {
    if !node.has_children() {
        return;
    }

    // lets get the children
    let mut vec = vec![];

    let mut child = node.first_child();
    loop {
        let child = match child {
            Some(c) => c,
            None => break,
        };

        vec.push(child);

        child = child.next_sibling();
    }

    // now, with the children, do magic.

    for child in vec {
        let node = child.value();
        match node {
            Element(el) => {
                match el.name() {
                    _ => el
                }
            }
            _ => {}
        }
    }
}

pub fn to_tomsg(s: &str) -> String {
    let mut frag = Html::parse_fragment(s);
    for child in frag.tree.root_mut().children() {}
}
*/
