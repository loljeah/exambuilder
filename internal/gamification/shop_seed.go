package gamification

import (
	"database/sql"
	"fmt"
)

// ShopItemSeed defines an item to seed
type ShopItemSeed struct {
	ID          string
	Name        string
	Description string
	Slot        string
	Price       int
	Rarity      string
	UnlockLevel int
}

// SeedShopItems populates the shop with all available items
func SeedShopItems(db *sql.DB) error {
	items := getAllShopItems()

	tx, err := db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	for _, item := range items {
		_, err := tx.Exec(`
			INSERT INTO shop_items (id, name, description, slot, price, rarity, unlock_level)
			VALUES (?, ?, ?, ?, ?, ?, ?)
			ON CONFLICT(id) DO UPDATE SET
				name = excluded.name,
				description = excluded.description,
				price = excluded.price,
				rarity = excluded.rarity,
				unlock_level = excluded.unlock_level
		`, item.ID, item.Name, item.Description, item.Slot, item.Price, item.Rarity, item.UnlockLevel)
		if err != nil {
			return fmt.Errorf("failed to seed item %s: %w", item.ID, err)
		}
	}

	return tx.Commit()
}

func getAllShopItems() []ShopItemSeed {
	var items []ShopItemSeed

	// HATS (30)
	items = append(items, []ShopItemSeed{
		// Common hats (5) - 50-100 coins, unlock level 1
		{"hat_baseball", "Baseball Cap", "A classic sporty cap", SlotHat, 50, RarityCommon, 1},
		{"hat_beanie", "Cozy Beanie", "Warm and stylish", SlotHat, 60, RarityCommon, 1},
		{"hat_beret", "Artist Beret", "Channel your inner artist", SlotHat, 70, RarityCommon, 1},
		{"hat_party", "Party Hat", "Ready to celebrate!", SlotHat, 80, RarityCommon, 1},
		{"hat_headphones", "Headphones", "Tune out and study", SlotHat, 100, RarityCommon, 1},

		// Uncommon hats (10) - 150-300 coins, unlock level 3-5
		{"hat_catears", "Cat Ears", "Nyaa~", SlotHat, 150, RarityUncommon, 3},
		{"hat_bunny_ears", "Bunny Ears", "Hop hop!", SlotHat, 150, RarityUncommon, 3},
		{"hat_cowboy", "Cowboy Hat", "Yeehaw, partner!", SlotHat, 180, RarityUncommon, 3},
		{"hat_bowler", "Bowler Hat", "Quite distinguished", SlotHat, 180, RarityUncommon, 4},
		{"hat_chef", "Chef Hat", "Cooking up knowledge", SlotHat, 200, RarityUncommon, 4},
		{"hat_tophat", "Top Hat", "Fancy and formal", SlotHat, 220, RarityUncommon, 4},
		{"hat_archer", "Archer Hat", "Robin Hood style", SlotHat, 250, RarityUncommon, 5},
		{"hat_fez", "Fez", "Fezzes are cool", SlotHat, 250, RarityUncommon, 5},
		{"hat_propeller", "Propeller Cap", "Ready for takeoff!", SlotHat, 280, RarityUncommon, 5},
		{"hat_flower_crown", "Flower Crown", "Spring vibes", SlotHat, 300, RarityUncommon, 5},

		// Rare hats (10) - 400-700 coins, unlock level 7-10
		{"hat_wizard", "Wizard Hat", "Arcane knowledge awaits", SlotHat, 400, RarityRare, 7},
		{"hat_pirate", "Pirate Hat", "Arr, knowledge be treasure!", SlotHat, 450, RarityRare, 7},
		{"hat_viking", "Viking Helm", "Conquer your exams!", SlotHat, 500, RarityRare, 8},
		{"hat_ninja", "Ninja Headband", "Stealthy studying", SlotHat, 500, RarityRare, 8},
		{"hat_detective", "Detective Hat", "Investigate the answers", SlotHat, 550, RarityRare, 8},
		{"hat_santa", "Santa Hat", "Ho ho ho!", SlotHat, 600, RarityRare, 9},
		{"hat_graduation", "Graduation Cap", "Academic excellence", SlotHat, 650, RarityRare, 9},
		{"hat_tiara", "Princess Tiara", "Royally smart", SlotHat, 650, RarityRare, 9},
		{"hat_mushroom", "Mushroom Cap", "Super powered!", SlotHat, 700, RarityRare, 10},
		{"hat_alien_antenna", "Alien Antenna", "Out of this world IQ", SlotHat, 700, RarityRare, 10},

		// Legendary hats (5) - 1000-2000 coins, unlock level 12-15
		{"hat_crown", "Royal Crown", "Ruler of knowledge", SlotHat, 1000, RarityLegendary, 12},
		{"hat_halo", "Angelic Halo", "Blessed with wisdom", SlotHat, 1200, RarityLegendary, 12},
		{"hat_jester", "Jester Hat", "Master of wit", SlotHat, 1500, RarityLegendary, 14},
		{"hat_unicorn_horn", "Unicorn Horn", "Magical intellect", SlotHat, 1800, RarityLegendary, 14},
		{"hat_space_helmet", "Space Helmet", "Infinite knowledge", SlotHat, 2000, RarityLegendary, 15},
	}...)

	// HELD ITEMS (30)
	items = append(items, []ShopItemSeed{
		// Common held (5) - 50-100 coins
		{"held_pencil", "Pencil", "The classic writing tool", SlotHeld, 50, RarityCommon, 1},
		{"held_book", "Study Book", "Knowledge in your hands", SlotHeld, 60, RarityCommon, 1},
		{"held_coffee", "Coffee Cup", "Fuel for studying", SlotHeld, 70, RarityCommon, 1},
		{"held_scroll", "Ancient Scroll", "Wisdom of the ages", SlotHeld, 80, RarityCommon, 1},
		{"held_feather", "Quill Feather", "Write with style", SlotHeld, 100, RarityCommon, 1},

		// Uncommon held (10) - 150-300 coins
		{"held_quill", "Writing Quill", "Elegant penmanship", SlotHeld, 150, RarityUncommon, 3},
		{"held_magnifier", "Magnifying Glass", "Examine closely", SlotHeld, 150, RarityUncommon, 3},
		{"held_compass", "Compass", "Find your direction", SlotHeld, 180, RarityUncommon, 3},
		{"held_dice", "Lucky Dice", "Fortune favors the bold", SlotHeld, 200, RarityUncommon, 4},
		{"held_key", "Mystery Key", "Unlock potential", SlotHeld, 220, RarityUncommon, 4},
		{"held_lantern", "Magic Lantern", "Light the way", SlotHeld, 250, RarityUncommon, 5},
		{"held_bell", "Golden Bell", "Ring in success", SlotHeld, 250, RarityUncommon, 5},
		{"held_paintbrush", "Paintbrush", "Create your future", SlotHeld, 280, RarityUncommon, 5},
		{"held_music_note", "Music Note", "Harmonious learning", SlotHeld, 280, RarityUncommon, 5},
		{"held_hourglass", "Hourglass", "Master of time", SlotHeld, 300, RarityUncommon, 5},

		// Rare held (10) - 400-700 coins
		{"held_wand", "Magic Wand", "Cast learning spells", SlotHeld, 400, RarityRare, 7},
		{"held_shield", "Guardian Shield", "Protect your knowledge", SlotHeld, 450, RarityRare, 7},
		{"held_sword", "Scholar's Sword", "Cut through confusion", SlotHeld, 500, RarityRare, 8},
		{"held_telescope", "Telescope", "See the big picture", SlotHeld, 500, RarityRare, 8},
		{"held_crystal_ball", "Crystal Ball", "Predict success", SlotHeld, 550, RarityRare, 8},
		{"held_controller", "Game Controller", "Gamify your learning", SlotHeld, 600, RarityRare, 9},
		{"held_potion_red", "Health Potion", "Restore your energy", SlotHeld, 600, RarityRare, 9},
		{"held_potion_blue", "Mana Potion", "Boost your magic", SlotHeld, 650, RarityRare, 9},
		{"held_potion_green", "XP Potion", "Enhance your gains", SlotHeld, 650, RarityRare, 10},
		{"held_heart", "Crystal Heart", "Love of learning", SlotHeld, 700, RarityRare, 10},

		// Legendary held (5) - 1000-2000 coins
		{"held_trophy", "Golden Trophy", "Champion of exams", SlotHeld, 1000, RarityLegendary, 12},
		{"held_gem_ruby", "Ruby Gem", "Burning passion", SlotHeld, 1200, RarityLegendary, 12},
		{"held_gem_sapphire", "Sapphire Gem", "Depth of wisdom", SlotHeld, 1400, RarityLegendary, 14},
		{"held_gem_emerald", "Emerald Gem", "Growth mindset", SlotHeld, 1600, RarityLegendary, 14},
		{"held_star", "Golden Star", "Stellar achievement", SlotHeld, 2000, RarityLegendary, 15},
	}...)

	// AURAS (12)
	items = append(items, []ShopItemSeed{
		// Common auras (2)
		{"aura_sparkles", "Sparkle Aura", "Glittering brilliance", SlotAura, 100, RarityCommon, 1},
		{"aura_bubbles", "Bubble Aura", "Light and floaty", SlotAura, 100, RarityCommon, 1},

		// Uncommon auras (4)
		{"aura_hearts", "Heart Aura", "Spread the love", SlotAura, 250, RarityUncommon, 4},
		{"aura_stars", "Star Aura", "Shine bright", SlotAura, 250, RarityUncommon, 4},
		{"aura_leaves", "Nature Aura", "One with nature", SlotAura, 300, RarityUncommon, 5},
		{"aura_snow", "Snow Aura", "Cool and calm", SlotAura, 300, RarityUncommon, 5},

		// Rare auras (4)
		{"aura_flames", "Flame Aura", "Burning determination", SlotAura, 500, RarityRare, 8},
		{"aura_lightning", "Lightning Aura", "Electric energy", SlotAura, 600, RarityRare, 9},
		{"aura_music", "Music Aura", "Feel the rhythm", SlotAura, 600, RarityRare, 9},
		{"aura_coins", "Coin Aura", "Wealth of knowledge", SlotAura, 700, RarityRare, 10},

		// Legendary auras (2)
		{"aura_rainbow", "Rainbow Aura", "Full spectrum genius", SlotAura, 1500, RarityLegendary, 14},
		{"aura_magic", "Magic Aura", "Mystical power", SlotAura, 2000, RarityLegendary, 15},
	}...)

	// BACKGROUNDS (30)
	items = append(items, []ShopItemSeed{
		// Common backgrounds (8) - 80-150 coins
		{"bg_starfield", "Starfield", "Study under the stars", SlotBackground, 80, RarityCommon, 1},
		{"bg_clouds", "Cloudy Sky", "Head in the clouds", SlotBackground, 80, RarityCommon, 1},
		{"bg_meadow", "Green Meadow", "Fresh and peaceful", SlotBackground, 100, RarityCommon, 1},
		{"bg_sunset", "Sunset View", "Golden hour learning", SlotBackground, 100, RarityCommon, 1},
		{"bg_rainy", "Rainy Day", "Cozy study weather", SlotBackground, 120, RarityCommon, 1},
		{"bg_autumn", "Autumn Forest", "Fall vibes", SlotBackground, 120, RarityCommon, 2},
		{"bg_beach", "Sunny Beach", "Vacation mode", SlotBackground, 150, RarityCommon, 2},
		{"bg_mountain", "Mountain View", "Peak performance", SlotBackground, 150, RarityCommon, 2},

		// Uncommon backgrounds (10) - 200-400 coins
		{"bg_cozy_room", "Cozy Room", "Home sweet study", SlotBackground, 200, RarityUncommon, 3},
		{"bg_library", "Library", "Classic study spot", SlotBackground, 220, RarityUncommon, 3},
		{"bg_garden", "Flower Garden", "Blooming intellect", SlotBackground, 250, RarityUncommon, 4},
		{"bg_forest", "Deep Forest", "Natural wisdom", SlotBackground, 250, RarityUncommon, 4},
		{"bg_ocean", "Ocean Depths", "Deep thoughts", SlotBackground, 280, RarityUncommon, 4},
		{"bg_desert", "Desert Dunes", "Oasis of knowledge", SlotBackground, 300, RarityUncommon, 5},
		{"bg_arctic", "Arctic Tundra", "Cool concentration", SlotBackground, 320, RarityUncommon, 5},
		{"bg_jungle", "Tropical Jungle", "Wild learning", SlotBackground, 350, RarityUncommon, 5},
		{"bg_cherry_blossom", "Cherry Blossoms", "Peaceful study", SlotBackground, 380, RarityUncommon, 6},
		{"bg_spring_meadow", "Spring Meadow", "Fresh beginnings", SlotBackground, 400, RarityUncommon, 6},

		// Rare backgrounds (8) - 500-800 coins
		{"bg_castle", "Castle", "Royal education", SlotBackground, 500, RarityRare, 7},
		{"bg_cave", "Crystal Cave", "Hidden gems", SlotBackground, 550, RarityRare, 8},
		{"bg_underwater", "Underwater", "Deep dive learning", SlotBackground, 600, RarityRare, 8},
		{"bg_volcanic", "Volcanic", "Explosive knowledge", SlotBackground, 650, RarityRare, 9},
		{"bg_cozy_fireplace", "Cozy Fireplace", "Warm and focused", SlotBackground, 700, RarityRare, 9},
		{"bg_enchanted", "Enchanted Forest", "Magical atmosphere", SlotBackground, 750, RarityRare, 10},
		{"bg_candy", "Candy Land", "Sweet success", SlotBackground, 750, RarityRare, 10},
		{"bg_neon_city", "Neon City", "Cyberpunk vibes", SlotBackground, 800, RarityRare, 10},

		// Legendary backgrounds (4) - 1200-2500 coins
		{"bg_space", "Outer Space", "Infinite learning", SlotBackground, 1200, RarityLegendary, 12},
		{"bg_galaxy", "Galaxy", "Cosmic wisdom", SlotBackground, 1500, RarityLegendary, 13},
		{"bg_crystal_cave", "Crystal Palace", "Brilliant minds", SlotBackground, 2000, RarityLegendary, 14},
		{"bg_mystical_portal", "Mystical Portal", "Gateway to knowledge", SlotBackground, 2500, RarityLegendary, 15},
	}...)

	return items
}
