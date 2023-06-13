use crate::{HtmxError, HtmxEvent, HtmxModifier};
use http::Uri;

pub struct HtmxTarget(String);
pub struct HtmxTrigger(HtmxEvent, Option<HtmxModifier>);

impl HtmxTrigger {
    pub fn new(event: impl ToString) -> Self {
        Self(HtmxEvent(event.to_string()), None)
    }
    pub fn modifier(mut self, modifier: impl ToString) -> Self {
        self.1 = Some(HtmxModifier(modifier.to_string()));
        self
    }
}

pub struct HtmxExt(String);

struct HtmxRequest(HtmxRequestType, String);
enum HtmxAttr {
    /// When the frontend makes a backend request
    Request(HtmxRequest),
    /// the hx-trigger attribute, see https://htmx.org/attributes/hx-trigger/
    Trigger(Vec<HtmxTrigger>),
    /// The hx-target attribute
    Target(HtmxTarget),
    ///.the hx-ext attribute
    Ext(HtmxExt),
    /// For eveything else -- just slaps the raw string into the attribute
    Misc(String),
}

impl TryInto<String> for HtmxTrigger {
    type Error = HtmxError;
    fn try_into(self) -> Result<String, HtmxError> {
        match self {
            HtmxTrigger(HtmxEvent(event), None) => Ok(event),
            HtmxTrigger(HtmxEvent(event), Some(HtmxModifier(modifier))) => {
                Ok(format!("{} {}", event, modifier))
            }
        }
    }
}

impl TryInto<String> for HtmxTarget {
    type Error = HtmxError;
    fn try_into(self) -> Result<String, HtmxError> {
        let target = self.0;
        Ok(format!("hx-target={}", target))
    }
}

impl TryInto<String> for HtmxRequest {
    type Error = HtmxError;
    fn try_into(self) -> Result<String, HtmxError> {
        let (req_type, uri) = (self.0, self.1);
        let _parse = Uri::try_from(&uri)?;
        Ok(format!("{}={}", req_type.to_string(), uri))
    }
}

impl TryInto<String> for HtmxExt {
    type Error = HtmxError;
    fn try_into(self) -> Result<String, HtmxError> {
        Ok(format!("hx-ext={}", self.0))
    }
}

impl TryInto<String> for HtmxAttr {
    type Error = HtmxError;
    fn try_into(self) -> Result<String, HtmxError> {
        match self {
            Self::Target(target) => target.try_into(),

            Self::Trigger(triggers) => {
                let strings: Result<Vec<String>, HtmxError> =
                    triggers.into_iter().map(|val| val.try_into()).collect();
                strings.map(|vector| {
                    let mut string = vector
                        .into_iter()
                        .fold(String::from("hx-trigger="), |wstr, val| {
                            format!("{}{},", wstr, val)
                        });
                    string.pop();
                    string
                })
            }
            Self::Request(req) => req.try_into(),
            Self::Ext(ext) => ext.try_into(),
            Self::Misc(misc) => Ok(misc),
        }
    }
}

#[derive(Default)]
pub struct HtmxAttrs {
    attrs: Vec<HtmxAttr>,
}

impl From<HtmxRequest> for HtmxAttr {
    fn from(value: HtmxRequest) -> Self {
        Self::Request(value)
    }
}

impl<S: AsRef<str>> From<S> for HtmxTrigger {
    fn from(value: S) -> Self {
        Self(HtmxEvent(value.as_ref().to_string()), None)
    }
}

enum HtmxRequestType {
    Get,
    Put,
    Post,
    Delete,
}

impl ToString for HtmxRequestType {
    fn to_string(&self) -> String {
        match self {
            Self::Get => "hx-get",
            Self::Put => "hx-put",
            Self::Post => "hx-post",
            Self::Delete => "hx-delete",
        }
        .into()
    }
}

impl HtmxAttrs {
    pub fn get(uri: impl ToString) -> Self {
        let req = HtmxRequest(HtmxRequestType::Get, uri.to_string());
        Self {
            attrs: vec![req.into()],
        }
    }
    pub fn post(uri: impl ToString) -> Self {
        let req = HtmxRequest(HtmxRequestType::Post, uri.to_string());
        Self {
            attrs: vec![req.into()],
        }
    }
    pub fn put(uri: impl ToString) -> Self {
        let req = HtmxRequest(HtmxRequestType::Put, uri.to_string());
        Self {
            attrs: vec![req.into()],
        }
    }

    pub fn delete(uri: impl ToString) -> Self {
        let req = HtmxRequest(HtmxRequestType::Delete, uri.to_string());
        Self {
            attrs: vec![req.into()],
        }
    }

    pub fn target(self, target: impl ToString) -> Self {
        let mut vec = self.attrs;
        vec.push(HtmxAttr::Target(HtmxTarget(target.to_string())));

        HtmxAttrs { attrs: vec }
    }
    pub fn extension(self, target: impl ToString) -> Self {
        let mut vec = self.attrs;
        vec.push(HtmxAttr::Ext(HtmxExt(target.to_string())));

        HtmxAttrs { attrs: vec }
    }
    pub fn triggers(self, triggers: impl IntoIterator<Item = impl Into<HtmxTrigger>>) -> Self {
        let triggers = triggers.into_iter().map(|val| val.into()).collect();
        let mut attrs = self.attrs;
        attrs.push(HtmxAttr::Trigger(triggers));
        HtmxAttrs { attrs }
    }
    pub fn trigger(self, trigger: impl Into<HtmxTrigger>) -> Self {
        let triggers = vec![trigger.into()];
        let mut attrs = self.attrs;
        attrs.push(HtmxAttr::Trigger(triggers));
        HtmxAttrs { attrs }
    }
    pub fn misc(self, misc: String) -> Self {
        let mut attrs = self.attrs;
        attrs.push(HtmxAttr::Misc(misc));
        HtmxAttrs { attrs }
    }
}

impl TryInto<String> for HtmxAttrs {
    type Error = HtmxError;
    fn try_into(self) -> Result<String, HtmxError> {
        let mut iter = self.attrs.into_iter();
        let mut string: String = iter.next().ok_or(HtmxError::EmptyAttrs)?.try_into()?;
        for attr in iter {
            let attr_str: String = attr.try_into()?;
            string = format!("{} {}", string, attr_str);
        }

        let rv = string.trim().to_string();

        Ok(rv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hx_get() {
        let demo: String = HtmxAttrs::get("/htmxdemo")
            .target("#repo")
            .triggers(["change", "load"])
            .try_into()
            .unwrap();
        let correct_str = format!("hx-get=/htmxdemo hx-target=#repo hx-trigger=change,load");
        assert_eq!(demo, correct_str)
    }
}
