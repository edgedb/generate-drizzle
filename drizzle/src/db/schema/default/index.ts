import { relations, sql } from 'drizzle-orm';
import { bigint, pgTable, smallint, text, uuid } from 'drizzle-orm/pg-core';


export const bookTable = pgTable("Book", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
  genre_id: uuid().references(() => genreTable.id),
  pages: smallint().notNull(),
  title: text().notNull(),
});

export const bookRelations = relations(bookTable, ({ one }) => ({
  genre: one(genreTable, {
    fields: [bookTable.genre_id],
    references: [genreTable.id],
  }),
}));

export const bookChaptersTable = pgTable("Book.chapters", {
  source_id: uuid().notNull().references(() => bookTable.id),
  target: text().notNull(),
});

export const bookChaptersRelations = relations(bookChaptersTable, ({ one }) => ({
  source: one(bookTable, {
    fields: [bookChaptersTable.source_id],
    references: [bookTable.id],
  }),
}));

export const contentTable = pgTable("Content", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
  genre_id: uuid().references(() => genreTable.id),
  title: text().notNull(),
});

export const contentRelations = relations(contentTable, ({ one }) => ({
  genre: one(genreTable, {
    fields: [contentTable.genre_id],
    references: [genreTable.id],
  }),
}));

export const contentSummaryTable = pgTable("ContentSummary", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
});

export const genreTable = pgTable("Genre", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
  name: text().notNull(),
});

export const movieTable = pgTable("Movie", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
  director_id: uuid().references(() => personTable.id),
  genre_id: uuid().references(() => genreTable.id),
  release_year: bigint({ mode: "number"}),
  title: text().notNull(),
});

export const movieRelations = relations(movieTable, ({ one }) => ({
  director: one(personTable, {
    fields: [movieTable.director_id],
    references: [personTable.id],
  }),
  genre: one(genreTable, {
    fields: [movieTable.genre_id],
    references: [genreTable.id],
  }),
}));

export const movieActorsTable = pgTable("Movie.actors", {
  source_id: uuid().notNull().references(() => movieTable.id),
  target_id: uuid().notNull().references(() => personTable.id),
});

export const movieActorsRelations = relations(movieActorsTable, ({ one }) => ({
  source: one(movieTable, {
    fields: [movieActorsTable.source_id],
    references: [movieTable.id],
  }),
  target: one(personTable, {
    fields: [movieActorsTable.target_id],
    references: [personTable.id],
  }),
}));

export const personTable = pgTable("Person", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
  first_name: text().notNull(),
  last_name: text(),
});

export const novelTable = pgTable("novel", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
  foo: text(),
  genre_id: uuid().references(() => genreTable.id),
  pages: smallint().notNull(),
  title: text().notNull(),
});

export const novelRelations = relations(novelTable, ({ one }) => ({
  genre: one(genreTable, {
    fields: [novelTable.genre_id],
    references: [genreTable.id],
  }),
}));

export const novelChaptersTable = pgTable("novel.chapters", {
  source_id: uuid().notNull().references(() => novelTable.id),
  target: text().notNull(),
});

export const novelChaptersRelations = relations(novelChaptersTable, ({ one }) => ({
  source: one(novelTable, {
    fields: [novelChaptersTable.source_id],
    references: [novelTable.id],
  }),
}));
