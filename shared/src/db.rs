use sqlx::postgres::PgPoolOptions;

use crate::models::{self, NewAuthor, NewPaper, NewSubject};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("database error")]
    Sqlx(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct DBConnection {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl DBConnection {
    pub async fn new(db_url: &str) -> Result<DBConnection> {
        Ok(DBConnection {
            pool: PgPoolOptions::new().connect(db_url).await?,
        })
    }

    pub async fn get_all_papers(&mut self) -> Result<Vec<models::Paper>> {
        sqlx::query_as!(models::Paper, "SELECT * FROM papers")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn count_papers(&mut self) -> Result<i64> {
        sqlx::query_scalar!("SELECT COUNT(*) FROM papers")
            .fetch_one(&self.pool)
            .await
            .map(|r| r.unwrap()) // TODO: not pretty
            .map_err(|e| e.into())
    }

    pub async fn get_paper(&mut self, desired_id: i32) -> Result<models::Paper> {
        sqlx::query_as!(
            models::Paper,
            "SELECT * FROM papers WHERE id = $1",
            desired_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.into())
    }

    pub async fn get_paper_authors(
        &mut self,
        desired_paper_id: i32,
    ) -> Result<Vec<models::Author>> {
        sqlx::query_as!(
            models::Author,
            "SELECT authors.id, authors.name
                FROM authors
            JOIN paper_author ON authors.id = paper_author.author_id
                WHERE paper_author.paper_id = $1",
            desired_paper_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.into())
    }

    pub async fn paper_exists(&mut self, desired_url: &str) -> Result<bool> {
        sqlx::query_scalar!(
            "SELECT EXISTS(SELECT * FROM papers WHERE url = $1)",
            desired_url
        )
        .fetch_one(&self.pool)
        .await
        .map(|r| r.unwrap()) // TODO: not pretty
        .map_err(|e| e.into())
    }

    pub async fn insert_paper(
        &self,
        new_paper: NewPaper,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<models::Id> {
        log::trace!("DB: inserting new paper {:?}", new_paper.url);
        Ok(sqlx::query_scalar!(
            "INSERT INTO papers (url, title, description, body)
                VALUES ($1, $2, $3, $4)
                RETURNING id",
            new_paper.url,
            new_paper.title,
            new_paper.description,
            new_paper.body,
        )
        .fetch_one(&mut **tx)
        .await?)
    }

    pub async fn insert_author(
        &self,
        new_author: NewAuthor,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<models::Id> {
        log::trace!("DB: inserting new author {:?}", new_author);

        sqlx::query!(
            "INSERT INTO authors (name)
                    VALUES ($1)
                    ON CONFLICT(name) DO NOTHING",
            new_author.name
        )
        .execute(&mut **tx)
        .await?;

        Ok(sqlx::query_scalar!(
            "SELECT authors.id FROM authors WHERE name = $1",
            new_author.name
        )
        .fetch_one(&mut **tx)
        .await?)
    }

    pub async fn insert_subject(
        &self,
        new_subject: NewSubject,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<models::Id> {
        log::trace!("DB: inserting new subject {:?}", new_subject.name);

        sqlx::query!(
            "INSERT INTO subjects (name)
                    VALUES ($1)
                    ON CONFLICT(name) DO NOTHING",
            new_subject.name
        )
        .execute(&mut **tx)
        .await?;

        Ok(sqlx::query_scalar!(
            "SELECT subjects.id FROM subjects WHERE name = $1",
            new_subject.name
        )
        .fetch_one(&mut **tx)
        .await?)
    }

    pub async fn set_paper_author(
        &mut self,
        paper_id: models::Id,
        author_id: models::Id,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<()> {
        log::trace!(
            "DB: inserting author {:?} for paper {:?}",
            author_id,
            paper_id
        );
        sqlx::query!(
            "INSERT INTO paper_author (paper_id, author_id)
                VALUES ($1, $2)",
            paper_id,
            author_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn set_paper_subject(
        &mut self,
        paper_id: models::Id,
        subject_id: models::Id,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<()> {
        log::trace!(
            "DB: inserting subject {:?} for paper {:?}",
            subject_id,
            paper_id
        );
        sqlx::query!(
            "INSERT INTO paper_subject (paper_id, subject_id) VALUES ($1, $2)",
            paper_id,
            subject_id,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn insert_paper_full(
        &mut self,
        paper: NewPaper,
        authors: Vec<NewAuthor>,
        subjects: Vec<NewSubject>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        let paper_id = self.insert_paper(paper, &mut tx).await?;

        for author in authors {
            let author_id = self.insert_author(author, &mut tx).await?;
            self.set_paper_author(paper_id, author_id, &mut tx).await?;
        }

        for subject in subjects {
            let subject_id = self.insert_subject(subject, &mut tx).await?;
            self.set_paper_subject(paper_id, subject_id, &mut tx).await?;
        }

        Ok(tx.commit().await?)
    }
}
