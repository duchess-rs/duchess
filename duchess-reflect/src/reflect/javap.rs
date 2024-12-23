use std::{collections::BTreeMap, path::Path, process::Command, sync::Arc};

use anyhow::{bail, Context};
use proc_macro2::Span;

use crate::{
    class_info::{ClassInfo, DotId},
    config::Configuration,
};

use super::{reflection_cache, JavapClassInfo, Reflect};

/// Reflector that uses JavaP to perform reflection
#[derive(Debug)]
pub struct JavapReflector {
    configuration: Configuration,
    classes: BTreeMap<DotId, Arc<JavapClassInfo>>,
}

impl Reflect for JavapReflector {
    fn reflect(&mut self, dot_id: &DotId, span: Span) -> syn::Result<Arc<JavapClassInfo>> {
        JavapReflector::reflect_and_cache(self, dot_id, span)
    }
}

impl JavapReflector {
    pub fn new(configuration: &Configuration) -> Self {
        Self {
            configuration: configuration.clone(),
            classes: BTreeMap::new(),
        }
    }

    fn serialize(&self) -> String {
        serde_json::to_string_pretty(&self.classes).expect("failed to serialize JSON")
    }

    pub fn dump_to(&self, out_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = reflection_cache(out_dir);
        let json = self.serialize();
        std::fs::write(&path, json)
            .with_context(|| format!("writing reflection cache data to {:?}", path))?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.classes.len()
    }

    fn reflect_via_javap(&self, class_name: &DotId, span: Span) -> anyhow::Result<JavapClassInfo> {
        let mut command = Command::new(self.configuration.bin_path("javap"));

        if let Some(classpath) = self.configuration.classpath() {
            command.arg("-cp").arg(classpath);
        }

        command.arg("-p").arg(format!("{}", class_name));

        let output_or_err = command.output();

        let output = match output_or_err {
            Ok(o) => o,
            Err(err) => {
                bail!("failed to execute `{command:?}`: {err}")
            }
        };

        if !output.status.success() {
            bail!(
                "unsuccessful execution of `{command:?}` (exit status: {}): {}",
                output.status,
                String::from_utf8(output.stderr).unwrap_or(String::from("error"))
            );
        }

        let s = match String::from_utf8(output.stdout) {
            Ok(o) => o,
            Err(err) => {
                bail!("failed to parse output of `{command:?}` as utf-8: {err}")
            }
        };

        let ci = ClassInfo::parse(&s, span)?;
        Ok(JavapClassInfo::from(ci))
    }

    fn reflect_and_cache(
        &mut self,
        class_name: &DotId,
        span: Span,
    ) -> syn::Result<Arc<JavapClassInfo>> {
        if let Some(ci) = self.classes.get(class_name) {
            return Ok(Arc::clone(ci));
        }

        let ci = self
            .reflect_via_javap(class_name, span)
            .map_err(|err| syn::Error::new(span, format!("{}", err)))?;

        let ci = Arc::new(ci);

        self.classes.insert(class_name.clone(), Arc::clone(&ci));

        Ok(ci)
    }
}
