import { relations, sql } from 'drizzle-orm';
import { pgSchema, text, uuid } from 'drizzle-orm/pg-core';


export const aTable = pgSchema("public::links").table("A", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
});

export const bTable = pgSchema("public::links").table("B", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
  prop_id: uuid().references(() => aTable.id),
});

export const bRelations = relations(bTable, ({ one }) => ({
  prop: one(aTable, {
    fields: [bTable.prop_id],
    references: [aTable.id],
  }),
}));

export const bATable = pgSchema("public::links").table("B.a", {
  source_id: uuid().notNull().references(() => bTable.id),
  target_id: uuid().notNull().references(() => aTable.id),
});

export const bARelations = relations(bATable, ({ one }) => ({
  source: one(bTable, {
    fields: [bATable.source_id],
    references: [bTable.id],
  }),
  target: one(aTable, {
    fields: [bATable.target_id],
    references: [aTable.id],
  }),
}));

export const bValsTable = pgSchema("public::links").table("B.vals", {
  source_id: uuid().notNull().references(() => bTable.id),
  target: text().notNull(),
});

export const bValsRelations = relations(bValsTable, ({ one }) => ({
  source: one(bTable, {
    fields: [bValsTable.source_id],
    references: [bTable.id],
  }),
}));

export const cTable = pgSchema("public::links").table("C", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
  prop_id: uuid().references(() => aTable.id),
});

export const cRelations = relations(cTable, ({ one }) => ({
  prop: one(aTable, {
    fields: [cTable.prop_id],
    references: [aTable.id],
  }),
}));

export const cValsTable = pgSchema("public::links").table("C.vals", {
  source_id: uuid().notNull().references(() => cTable.id),
  target: text().notNull(),
});

export const cValsRelations = relations(cValsTable, ({ one }) => ({
  source: one(cTable, {
    fields: [cValsTable.source_id],
    references: [cTable.id],
  }),
}));

export const cATable = pgSchema("public::links").table("C.a", {
  source_id: uuid().notNull().references(() => cTable.id),
  target_id: uuid().notNull().references(() => aTable.id),
});

export const cARelations = relations(cATable, ({ one }) => ({
  source: one(cTable, {
    fields: [cATable.source_id],
    references: [cTable.id],
  }),
  target: one(aTable, {
    fields: [cATable.target_id],
    references: [aTable.id],
  }),
}));
