use diesel::prelude::*;

pub type Id = i32;

#[derive(Identifiable, Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::papers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Paper {
    pub id: Id,
    pub title: String,
    pub description: String,
    pub body: String,
}

#[derive(Identifiable, Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::authors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Author {
    pub id: Id,
    pub name: String,
}

#[derive(Identifiable, Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::categories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Category {
    pub id: Id,
    pub name: String,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Insertable, Debug)]
#[diesel(table_name = crate::schema::paper_author)]
#[diesel(belongs_to(Paper))]
#[diesel(belongs_to(Author))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(paper_id, author_id))]
pub struct PaperAuthor {
    pub paper_id: Id,
    pub author_id: Id,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Insertable, Debug)]
#[diesel(table_name = crate::schema::paper_category)]
#[diesel(belongs_to(Paper))]
#[diesel(belongs_to(Category))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(paper_id, category_id))]
pub struct PaperCategory {
    pub paper_id: Id,
    pub category_id: Id,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::papers)]
pub struct NewPaper<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub description: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::authors)]
pub struct NewAuthor<'a> {
    pub name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::categories)]
pub struct NewCategory<'a> {
    pub name: &'a str,
}
