use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Article::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Article::ArticleId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Article::Title).string())
                    .col(ColumnDef::new(Article::Url).string().unique_key())
                    .col(ColumnDef::new(Article::Markdown).string())
                    .col(ColumnDef::new(Article::Content).string())
                    .col(ColumnDef::new(Article::Excerpt).string())
                    .col(ColumnDef::new(Article::Created).integer().not_null())
                    .col(ColumnDef::new(Article::Updated).integer())
                    .col(ColumnDef::new(Article::Type).integer())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tag::TagId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tag::TagValue).text().unique_key())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ArticleTag::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ArticleTag::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ArticleTag::ArticleId).integer())
                    .col(ColumnDef::new(ArticleTag::TagId).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .from(ArticleTag::Table, ArticleTag::ArticleId)
                            .to(Article::Table, Article::ArticleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ArticleTag::Table, ArticleTag::TagId)
                            .to(Tag::Table, Tag::TagId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Article::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ArticleTag::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Article {
    Table,
    ArticleId,
    Title,
    Url,
    Markdown,
    Excerpt,
    Content,
    Created,
    Updated,
    Type,
}

#[derive(DeriveIden)]
enum Tag {
    Table,
    TagId,
    TagValue,
}

#[derive(DeriveIden)]
enum ArticleTag {
    Table,
    Id,
    ArticleId,
    TagId,
}
