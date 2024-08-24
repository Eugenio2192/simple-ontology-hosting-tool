use std::borrow::Borrow;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::rc::Rc;

use horned_owl::error::HornedError;
use horned_owl::io::{ParserConfiguration, ParserOutput, RDFParserConfiguration};
use horned_owl::model::{
    Build, ClassExpression, Component, MutableOntology, RcAnnotatedComponent, SubClassOf, IRI,
};
use horned_owl::ontology::component_mapped::RcComponentMappedOntology;
use horned_owl::ontology::iri_mapped::RcIRIMappedOntology;
use horned_owl::ontology::set::SetOntology;
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn split_ontology(file: &str, pool: &SqlitePool) -> Result<(), HornedError> {
    let file = File::open(file)?;
    let mut bufreader = BufReader::new(file);
    let config = ParserConfiguration {
        rdf: RDFParserConfiguration { lax: true },
        ..Default::default()
    };
    let b = Build::new_rc();
    let parser_output: ParserOutput<Rc<str>, RcAnnotatedComponent> = ParserOutput::rdf(
        horned_owl::io::rdf::reader::read_with_build(&mut bufreader, &b, config)?,
    );
    let set_ont: SetOntology<Rc<str>> = parser_output.into();
    let declarations: RcComponentMappedOntology = set_ont.clone().into();
    let declaration_iris: Vec<IRI<Rc<str>>> = declarations
        .index()
        .declare_class()
        .map(|dc| dc.borrow().0 .0.clone())
        .collect();
    let mut iri_mapped: RcIRIMappedOntology = set_ont.into();
    for dec in declaration_iris.iter() {
        let mut o = RcComponentMappedOntology::new_rc();
        for comp in iri_mapped.components_for_iri(dec) {
            match &comp.component {
                Component::SubClassOf(SubClassOf { sup, sub: _ }) => match sup {
                    ClassExpression::Class(c) => {
                        if c.0 == *dec {
                        } else {
                            o.insert(comp.clone());
                        }
                    }
                    _ => {
                        o.insert(comp.clone());
                    }
                },
                _ => {
                    o.insert(comp.clone());
                }
            }
        }
        let mut bw = BufWriter::new(Vec::<u8>::new());
        // horned_owl::io::rdf::writer::write(&mut file, &o);
        horned_owl::io::owx::writer::write(&mut bw, &o, None)?;
        let uuid = Uuid::new_v4();
        let content = bw.into_inner().unwrap();
        let name = dec
            .to_string()
            .split("/")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .last()
            .unwrap()
            .clone();
        sqlx::query!(
            "INSERT INTO xml_cache (id, name, content)
                    VALUES ($1, $2, $3)",
            uuid,
            name,
            content
        )
        .execute(pool)
        .await
        .expect("Failed to store content.");
    }
    Ok(())
}
